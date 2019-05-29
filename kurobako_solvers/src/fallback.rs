// TODO: remove
use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::{
    uniform, Categorical, Condition, Conditional, Continuous, Discrete, Distribution, ParamDomain,
    ParamValue, Unconditional,
};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{
    ObservedObs, Solver, SolverCapabilities, SolverCapability, SolverRecipe, SolverSpec,
    UnobservedObs,
};
use kurobako_core::{ErrorKind, Result};
use rand::distributions::Distribution as _;
use rand::{self, Rng};
use rustats::range::Range;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::iter;
use yamakan::budget::Budgeted;
use yamakan::observation::{IdGen, ObsId};

#[derive(Debug)]
pub struct FallbackSolver<T> {
    inner: T,
    adaptors: Vec<Adaptor>,
    obss: HashMap<ObsId, Vec<ParamValue>>,
}
impl<T: Solver> FallbackSolver<T> {
    pub fn new<R>(recipe: R, problem: ProblemSpec) -> Result<Self>
    where
        R: SolverRecipe<Solver = T>,
    {
        let required_capabilities = problem.required_solver_capabilities();
        let mut inner = track!(recipe.create_solver(problem.clone()))?;
        let actual_capabilities = inner.specification().capabilities;

        let mut adaptors = Vec::new();
        let incapables = actual_capabilities.incapables(&required_capabilities);
        if !incapables.is_empty() {
            debug!("Incapables: {:?}", incapables);

            let adjuster = ProblemSpecAdjuster::new(incapables, &problem);
            let adjusted = track!(adjuster.adjust())?;

            trace!("Actual domain: {:?}", problem.params_domain);
            trace!("Adjusted domain: {:?}", adjusted.0);

            inner = track!(recipe.create_solver(adjusted.0))?;
            adaptors = adjusted.1;
        }

        Ok(Self {
            inner,
            adaptors,
            obss: HashMap::new(),
        })
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}
impl<T: Solver> Solver for FallbackSolver<T> {
    fn specification(&self) -> SolverSpec {
        let mut spec = self.inner.specification();
        spec.capabilities = SolverCapabilities::all();
        spec
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        let mut obs = track!(self.inner.ask(rng, idg))?;
        if !self.adaptors.is_empty() {
            let mut actual_params = VecDeque::from(obs.param.get().clone());
            let mut adjusted_params = Vec::new();
            for a in &self.adaptors {
                track!(a.backword(&mut actual_params, &mut adjusted_params))?;
            }

            let actual_params = obs.param.get().clone();
            obs = obs.map_param(|p| Budgeted::new(p.budget(), adjusted_params));
            self.obss.insert(obs.id, actual_params);
        }
        Ok(obs)
    }

    fn tell(&mut self, mut obs: ObservedObs) -> Result<()> {
        // TODO: remove from obss if it finish
        if !self.adaptors.is_empty() {
            let actual_params = if let Some(actual) = self.obss.get(&obs.id).cloned() {
                actual
            } else {
                let mut adjusted_params = VecDeque::from(obs.param.get().clone());
                let mut actual_params = Vec::new();
                for a in &self.adaptors {
                    track!(a.forward(&mut adjusted_params, &mut actual_params))?;
                }
                actual_params
            };
            obs = obs.map_param(|p| Budgeted::new(p.budget(), actual_params));
        }
        track!(self.inner.tell(obs))
    }
}

#[derive(Debug)]
struct ProblemSpecAdjuster {
    incapables: SolverCapabilities,
    problem: ProblemSpec,
    base_problem: ProblemSpec,
    adaptors: Vec<Adaptor>,
}
impl ProblemSpecAdjuster {
    fn new(incapables: SolverCapabilities, base_problem: &ProblemSpec) -> Self {
        Self {
            incapables,
            problem: ProblemSpec {
                params_domain: Vec::new(),
                values_domain: Vec::new(),
                ..base_problem.clone()
            },
            base_problem: base_problem.clone(),
            adaptors: Vec::new(),
        }
    }

    fn adjust(mut self) -> Result<(ProblemSpec, Vec<Adaptor>)> {
        if self.incapables.contains(SolverCapability::MultiObjective) {
            track_panic!(ErrorKind::Other, "Unimplemented: {:?}");
        } else {
            self.problem.values_domain = self.base_problem.values_domain.clone();
        }

        for param_domain in self.base_problem.params_domain.clone() {
            let adaptor = track!(self.adjust_param_domain(&param_domain))?;
            self.problem.params_domain.extend(adaptor.adjusted_params());
            self.adaptors.push(adaptor);
        }

        Ok((self.problem, self.adaptors))
    }

    fn adjust_param_domain(&self, param_domain: &ParamDomain) -> Result<Adaptor> {
        match param_domain {
            ParamDomain::Continuous(p)
                if p.distribution == Distribution::LogUniform
                    && self.incapables.contains(SolverCapability::LogUniform) =>
            {
                track!(LogUniformAdaptor::new(p)).map(Adaptor::LogUniform)
            }
            ParamDomain::Discrete(p) if self.incapables.contains(SolverCapability::Discrete) => {
                track!(DiscreteAdaptor::new(p)).map(Adaptor::Discrete)
            }
            ParamDomain::Categorical(p)
                if self.incapables.contains(SolverCapability::Categorical) =>
            {
                Ok(Adaptor::Categorical(CategoricalAdaptor::new(p)))
            }
            ParamDomain::Conditional(p) => {
                let adaptor = track!(self.adjust_param_domain(&p.param.to_domain()))?;
                Ok(Adaptor::Conditional(ConditionalAdaptor::new(
                    self.incapables.contains(SolverCapability::Conditional),
                    &p.condition,
                    &p.param,
                    adaptor,
                )))
            }
            _ => Ok(Adaptor::Noop(param_domain.clone())),
        }
    }
}

#[derive(Debug)]
enum Adaptor {
    Noop(ParamDomain),
    Discrete(DiscreteAdaptor),
    Categorical(CategoricalAdaptor),
    LogUniform(LogUniformAdaptor),
    Conditional(ConditionalAdaptor),
}
impl Adaptor {
    fn adjusted_params<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = ParamDomain>> {
        match self {
            Adaptor::Noop(p) => Box::new(iter::once(p.clone())),
            Adaptor::Discrete(a) => a.adjusted_params(),
            Adaptor::Categorical(a) => a.adjusted_params(),
            Adaptor::LogUniform(a) => a.adjusted_params(),
            Adaptor::Conditional(a) => a.adjusted_params(),
        }
    }

    fn backword(
        &self,
        actual: &mut VecDeque<ParamValue>,
        adjusted: &mut Vec<ParamValue>,
    ) -> Result<()> {
        match self {
            Adaptor::Noop(_) => {
                adjusted.extend(actual.pop_front());
                Ok(())
            }
            Adaptor::Discrete(a) => track!(a.backword(actual, adjusted)),
            Adaptor::Categorical(a) => track!(a.backword(actual, adjusted)),
            Adaptor::LogUniform(a) => track!(a.backword(actual, adjusted)),
            Adaptor::Conditional(a) => track!(a.backword(actual, adjusted)),
        }
    }

    fn forward(
        &self,
        adjusted: &mut VecDeque<ParamValue>,
        actual: &mut Vec<ParamValue>,
    ) -> Result<()> {
        match self {
            Adaptor::Noop(_) => {
                actual.extend(adjusted.pop_front());
                Ok(())
            }
            Adaptor::Discrete(a) => track!(a.forward(adjusted, actual)),
            Adaptor::Categorical(a) => track!(a.forward(adjusted, actual)),
            Adaptor::LogUniform(a) => track!(a.forward(adjusted, actual)),
            Adaptor::Conditional(a) => track!(a.forward(adjusted, actual)),
        }
    }
}

#[derive(Debug)]
struct DiscreteAdaptor {
    adjusted: ParamDomain,
}
impl DiscreteAdaptor {
    fn new(actual: &Discrete) -> Result<Self> {
        let adjusted = track!(uniform(
            &actual.name,
            actual.range.low as f64,
            actual.range.high as f64
        ))?;
        Ok(Self { adjusted })
    }

    fn adjusted_params<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = ParamDomain>> {
        Box::new(iter::once(self.adjusted.clone()))
    }

    fn backword(
        &self,
        actual: &mut VecDeque<ParamValue>,
        adjusted: &mut Vec<ParamValue>,
    ) -> Result<()> {
        let v = track_assert_some!(actual.pop_front(), ErrorKind::Bug);
        let v = v.try_map(|v| {
            let v = track_assert_some!(v.as_continuous(), ErrorKind::Bug; v);
            Ok(ParamValue::Discrete(v.get().floor() as i64))
        })?;
        adjusted.push(v);
        Ok(())
    }

    fn forward(
        &self,
        adjusted: &mut VecDeque<ParamValue>,
        actual: &mut Vec<ParamValue>,
    ) -> Result<()> {
        let v = track_assert_some!(adjusted.pop_front(), ErrorKind::Bug);
        let v = v.try_map(|v| {
            let v = track_assert_some!(v.as_discrete(), ErrorKind::Bug; v);
            let n = v as f64 + rand::thread_rng().gen_range(0.0, 1.0);
            let n = track!(FiniteF64::new(n))?;
            Ok(ParamValue::Continuous(n))
        })?;
        actual.push(v);
        Ok(())
    }
}

#[derive(Debug)]
struct CategoricalAdaptor {
    choices: usize,
    adjusted: Vec<ParamDomain>,
}
impl CategoricalAdaptor {
    fn new(actual: &Categorical) -> Self {
        let adjusted = actual
            .choices
            .iter()
            .map(|c| {
                uniform(&format!("{}.{}", actual.name, c), 0.0, 1.0)
                    .unwrap_or_else(|e| unreachable!("{}", e))
            })
            .collect::<Vec<_>>();
        Self {
            choices: actual.choices.len(),
            adjusted,
        }
    }

    fn adjusted_params<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = ParamDomain>> {
        Box::new(self.adjusted.iter().cloned())
    }

    fn backword(
        &self,
        actual: &mut VecDeque<ParamValue>,
        adjusted: &mut Vec<ParamValue>,
    ) -> Result<()> {
        let (index, v, is_conditional) = track_assert_some!(
            actual
                .drain(..self.choices)
                .enumerate()
                .map(|(i, p)| match p {
                    ParamValue::Continuous(v) => (i, Some(v), false),
                    ParamValue::Conditional(None) => (i, None, true),
                    ParamValue::Conditional(Some(v)) => {
                        if let ParamValue::Continuous(v) = *v {
                            (i, Some(v), true)
                        } else {
                            unreachable!()
                        }
                    }
                    _ => unreachable!(),
                })
                .max_by_key(|t| t.1),
            ErrorKind::Bug
        );
        if is_conditional && v.is_none() {
            adjusted.push(ParamValue::Conditional(None));
        } else if is_conditional {
            adjusted.push(ParamValue::Conditional(Some(Box::new(
                ParamValue::Categorical(index),
            ))));
        } else {
            adjusted.push(ParamValue::Categorical(index));
        }
        Ok(())
    }

    fn forward(
        &self,
        adjusted: &mut VecDeque<ParamValue>,
        actual: &mut Vec<ParamValue>,
    ) -> Result<()> {
        let v = track_assert_some!(adjusted.pop_front(), ErrorKind::Bug);
        let (index, is_conditional) = match v {
            ParamValue::Categorical(index) => (Some(index), false),
            ParamValue::Conditional(None) => (None, true),
            ParamValue::Conditional(Some(v)) => {
                if let ParamValue::Categorical(index) = *v {
                    (Some(index), true)
                } else {
                    track_panic!(ErrorKind::Bug; v);
                }
            }
            _ => {
                track_panic!(ErrorKind::Bug; v);
            }
        };
        if let Some(index) = index {
            let choices = (0..self.choices)
                .map(|i| unsafe { FiniteF64::new_unchecked(if i == index { 1.0 } else { 0.0 }) })
                .map(|v| {
                    if is_conditional {
                        ParamValue::Conditional(Some(Box::new(ParamValue::Continuous(v))))
                    } else {
                        ParamValue::Continuous(v)
                    }
                });
            actual.extend(choices)
        } else {
            actual.extend((0..self.choices).map(|_| ParamValue::Conditional(None)));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct LogUniformAdaptor {
    low: FiniteF64,
    adjusted: ParamDomain,
}
impl LogUniformAdaptor {
    fn new(actual: &Continuous) -> Result<Self> {
        let mut adjusted = actual.clone();

        let low = track!(FiniteF64::new(1.0))?;
        let high = track!(FiniteF64::new(
            (actual.range.high.get() - actual.range.low.get()).exp()
        ))?;
        adjusted.range = track!(Range::new(low, high))?;
        adjusted.distribution = Distribution::Uniform;

        Ok(Self {
            low: actual.range.low,
            adjusted: ParamDomain::Continuous(adjusted),
        })
    }

    fn adjusted_params<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = ParamDomain>> {
        Box::new(iter::once(self.adjusted.clone()))
    }

    fn backword(
        &self,
        actual: &mut VecDeque<ParamValue>,
        adjusted: &mut Vec<ParamValue>,
    ) -> Result<()> {
        let v = track_assert_some!(actual.pop_front(), ErrorKind::Bug);
        let v = v.try_map(|v| {
            let v = track_assert_some!(v.as_continuous(), ErrorKind::Bug; v);
            let v = track!(FiniteF64::new(v.get().ln() + self.low.get()))?;
            Ok(ParamValue::Continuous(v))
        })?;
        adjusted.push(v);
        Ok(())
    }

    fn forward(
        &self,
        adjusted: &mut VecDeque<ParamValue>,
        actual: &mut Vec<ParamValue>,
    ) -> Result<()> {
        let v = track_assert_some!(adjusted.pop_front(), ErrorKind::Bug);
        let v = v.try_map(|v| {
            let v = track_assert_some!(v.as_continuous(), ErrorKind::Bug; v);
            let v = track!(FiniteF64::new((v.get() - self.low.get()).exp()))?;
            Ok(ParamValue::Continuous(v))
        })?;
        actual.push(v);
        Ok(())
    }
}

#[derive(Debug)]
struct ConditionalAdaptor {
    do_unwrap: bool,
    condition: Condition,
    actual: Unconditional,
    inner: Box<Adaptor>,
}
impl ConditionalAdaptor {
    fn new(do_unwrap: bool, condition: &Condition, actual: &Unconditional, inner: Adaptor) -> Self {
        ConditionalAdaptor {
            do_unwrap,
            condition: condition.clone(),
            actual: actual.clone(),
            inner: Box::new(inner),
        }
    }

    fn adjusted_params<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = ParamDomain>> {
        Box::new(self.inner.adjusted_params().map(move |p| {
            if self.do_unwrap {
                p
            } else {
                ParamDomain::Conditional(Conditional {
                    condition: self.condition.clone(),
                    param: Box::new(
                        track!(Unconditional::try_from(p))
                            .unwrap_or_else(|e| unreachable!("{}", e)),
                    ),
                })
            }
        }))
    }

    fn backword(
        &self,
        actual: &mut VecDeque<ParamValue>,
        adjusted: &mut Vec<ParamValue>,
    ) -> Result<()> {
        track!(self.inner.backword(actual, adjusted))?;
        if self.do_unwrap {
            // TODO: check condition
            let v = track_assert_some!(adjusted.pop(), ErrorKind::Bug);
            adjusted.push(ParamValue::Conditional(Some(Box::new(v))));
        }
        Ok(())
    }

    fn forward(
        &self,
        adjusted: &mut VecDeque<ParamValue>,
        actual: &mut Vec<ParamValue>,
    ) -> Result<()> {
        if self.do_unwrap {
            if let Some(ParamValue::Conditional(v)) = adjusted.pop_front() {
                if let Some(v) = v {
                    adjusted.push_front((*v).clone());
                } else {
                    let v = self.actual.sample(&mut rand::thread_rng());
                    adjusted.push_front(v);
                }
            } else {
                track_panic!(ErrorKind::Bug);
            }
        }
        track!(self.inner.forward(adjusted, actual))?;
        Ok(())
    }
}
