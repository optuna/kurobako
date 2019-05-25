use kurobako_core::Result;
use std::cell::RefCell;
use std::cmp;
use std::io::Write;
use std::rc::Rc;

#[derive(Debug)]
pub struct MarkdownWriter<'a, W> {
    inner: Rc<RefCell<&'a mut W>>,
    level: usize,
}
impl<'a, W: Write> MarkdownWriter<'a, W> {
    pub fn new(inner: &'a mut W) -> Self {
        Self {
            inner: Rc::new(RefCell::new(inner)),
            level: 0,
        }
    }

    pub fn heading(&mut self, s: &str) -> Result<Self> {
        for _ in 0..=self.level {
            track_any_err!(write!(self.inner.borrow_mut(), "#"))?;
        }
        track_any_err!(writeln!(self.inner.borrow_mut(), " {}\n", s))?;

        Ok(Self {
            inner: self.inner.clone(),
            level: self.level + 1,
        })
    }

    pub fn writeln(&mut self, s: &str) -> Result<()> {
        track_any_err!(writeln!(self.inner.borrow_mut(), "{}", s))?;
        Ok(())
    }

    pub fn newline(&mut self) -> Result<()> {
        track_any_err!(writeln!(self.inner.borrow_mut()))?;
        Ok(())
    }

    pub fn write(&mut self, s: &str) -> Result<()> {
        track_any_err!(write!(self.inner.borrow_mut(), "{}", s))?;
        Ok(())
    }

    pub fn write_table(&mut self, table: &Table) -> Result<()> {
        let mut widthes = table
            .headers
            .iter()
            .map(|h| h.name.len())
            .collect::<Vec<_>>();

        for col in 0..table.headers.len() {
            for row in &table.rows {
                if let Some(item) = row.items.get(col) {
                    widthes[col] = cmp::max(widthes[col], item.len());
                }
            }
        }

        track!(self.write("|"))?;
        for (h, w) in table.headers.iter().zip(widthes.iter().cloned()) {
            let s = match h.align {
                Align::Left => format!(" {:<width$} |", h.name, width = w),
                Align::Center => format!(" {:^width$} |", h.name, width = w),
                Align::Right => format!(" {:>width$} |", h.name, width = w),
            };
            track!(self.write(&s))?;
        }
        track!(self.newline())?;

        track!(self.write("|"))?;
        for (h, w) in table.headers.iter().zip(widthes.iter().cloned()) {
            let s = match h.align {
                Align::Left => format!(":{:-<width$}-|", "-", width = w),
                Align::Center => format!(":{:-^width$}:|", "-", width = w),
                Align::Right => format!("-{:->width$}:|", "-", width = w),
            };
            track!(self.write(&s))?;
        }
        track!(self.newline())?;

        for row in &table.rows {
            track!(self.write("|"))?;
            for (h, (item, w)) in table
                .headers
                .iter()
                .zip(row.items.iter().zip(widthes.iter().cloned()))
            {
                let s = match h.align {
                    Align::Left => format!(" {:<width$} |", item, width = w),
                    Align::Center => format!(" {:^width$} |", item, width = w),
                    Align::Right => format!(" {:>width$} |", item, width = w),
                };
                track!(self.write(&s))?;
            }
            track!(self.newline())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Table {
    headers: Vec<ColumnHeader>,
    rows: Vec<Row>,
}
impl Table {
    pub fn new<I>(headers: I) -> Self
    where
        I: Iterator<Item = ColumnHeader>,
    {
        let headers = headers.collect();
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    pub fn row(&mut self) -> &mut Row {
        self.rows.push(Row::default());
        self.rows.last_mut().unwrap_or_else(|| unreachable!())
    }
}

#[derive(Debug)]
pub struct ColumnHeader {
    name: String,
    align: Align,
}
impl ColumnHeader {
    pub fn new(name: &str, align: Align) -> Self {
        Self {
            name: name.to_owned(),
            align,
        }
    }
}

#[derive(Debug, Default)]
pub struct Row {
    items: Vec<String>,
}
impl Row {
    pub fn item<T>(&mut self, item: T) -> &mut Self
    where
        T: ToString,
    {
        self.items.push(item.to_string());
        self
    }
}

#[derive(Debug)]
pub enum Align {
    Left,
    Center,
    Right,
}
