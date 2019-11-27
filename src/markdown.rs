use kurobako_core::Result;
use std::cmp;
use std::io::Write;

#[derive(Debug)]
pub struct MarkdownWriter<'a, W> {
    writer: &'a mut W,
    level: usize,
}
impl<'a, W: Write> MarkdownWriter<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self::with_level(writer, 0)
    }

    pub fn with_level(writer: &'a mut W, level: usize) -> Self {
        Self { writer, level }
    }

    pub fn heading(&mut self, s: &str) -> Result<MarkdownWriter<W>> {
        for _ in 0..=self.level {
            track_write!(self.writer, "#")?
        }
        track_writeln!(self.writer, " {}\n", s)?;

        Ok(MarkdownWriter {
            writer: self.writer,
            level: self.level + 1,
        })
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn newline(&mut self) -> Result<()> {
        track_writeln!(self.writer)
    }

    pub fn code_block(&mut self, lang: &str, code: &str) -> Result<()> {
        track_writeln!(self.writer, "```{}", lang)?;
        track_writeln!(self.writer, "{}", code)?;
        track_writeln!(self.writer, "```")?;
        Ok(())
    }

    pub fn list(&mut self) -> ListWriter<&mut W> {
        ListWriter::new(&mut self.writer)
    }

    pub fn write_table(&mut self, table: &Table) -> Result<()> {
        let mut widthes = table
            .headers
            .iter()
            .map(|h| h.name.len())
            .collect::<Vec<_>>();

        #[allow(clippy::needless_range_loop)]
        for col in 0..table.headers.len() {
            for row in &table.rows {
                if let Some(item) = row.items.get(col) {
                    widthes[col] = cmp::max(widthes[col], item.len());
                }
            }
        }

        track_write!(self.writer, "|")?;
        for (h, w) in table.headers.iter().zip(widthes.iter().cloned()) {
            let s = match h.align {
                Align::Left => format!(" {:<width$} |", h.name, width = w),
                Align::Center => format!(" {:^width$} |", h.name, width = w),
                Align::Right => format!(" {:>width$} |", h.name, width = w),
            };
            track_write!(self.writer, "{}", s)?;
        }
        track!(self.newline())?;

        track_write!(self.writer, "|")?;
        for (h, w) in table.headers.iter().zip(widthes.iter().cloned()) {
            let s = match h.align {
                Align::Left => format!(":{:-<width$}-|", "-", width = w),
                Align::Center => format!(":{:-^width$}:|", "-", width = w),
                Align::Right => format!("-{:->width$}:|", "-", width = w),
            };
            track_write!(self.writer, "{}", s)?;
        }
        track!(self.newline())?;

        for row in &table.rows {
            track_write!(self.writer, "|")?;
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
                track_write!(self.writer, "{}", s)?;
            }
            track!(self.newline())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ListWriter<W> {
    writer: W,
    level: usize,
    number: Option<usize>,
}
impl<W: Write> ListWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            level: 0,
            number: None,
        }
    }

    pub fn numbered(mut self) -> Self {
        self.number = Some(1);
        self
    }

    pub fn item(&mut self, s: &str) -> Result<()> {
        for _ in 0..self.level {
            track_write!(self.writer, "  ")?;
        }
        if let Some(n) = self.number.as_mut() {
            track_writeln!(self.writer, "{}. {}", n, s)?;
            *n += 1;
        } else {
            track_writeln!(self.writer, "- {}", s)?;
        }
        Ok(())
    }

    pub fn list(&mut self) -> ListWriter<&mut W> {
        ListWriter {
            writer: &mut self.writer,
            level: self.level + 1,
            number: None,
        }
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
