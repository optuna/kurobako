use kurobako_core::Result;
use std::io::Write;

#[derive(Debug)]
pub struct MarkdownWriter<W> {
    writer: W,
    level: usize,
}
impl<W: Write> MarkdownWriter<W> {
    pub fn new(writer: W) -> Self {
        Self::with_level(writer, 0)
    }

    pub fn with_level(writer: W, level: usize) -> Self {
        Self { writer, level }
    }

    pub fn heading(&mut self, s: &str) -> Result<MarkdownWriter<&mut W>> {
        for _ in 0..=self.level {
            track_write!(self.writer, "#")?
        }
        track_writeln!(self.writer, " {}\n", s)?;

        Ok(MarkdownWriter {
            writer: &mut self.writer,
            level: self.level + 1,
        })
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn newline(&mut self) -> Result<()> {
        track_writeln!(self.writer)
    }

    pub fn list(&mut self) -> ListWriter<&mut W> {
        ListWriter::new(&mut self.writer)
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

    pub fn numbered_list(&mut self) -> ListWriter<&mut W> {
        ListWriter {
            writer: &mut self.writer,
            level: self.level + 1,
            number: Some(1),
        }
    }
}

//         for _ in 0..=self.level {
//             track_any_err!(write!(self.inner.borrow_mut(), "#"))?;
//         }
//         track_any_err!(writeln!(self.inner.borrow_mut(), " {}\n", s))?;

//         Ok(Self {
//             inner: self.inner.clone(),
//             level: self.level + 1,
//         })
//     }

//     pub fn writeln(&mut self, s: &str) -> Result<()> {
//         track_any_err!(writeln!(self.inner.borrow_mut(), "{}", s))?;
//         Ok(())
//     }

//     pub fn newline(&mut self) -> Result<()> {
//         track_any_err!(writeln!(self.inner.borrow_mut()))?;
//         Ok(())
//     }

//     pub fn write(&mut self, s: &str) -> Result<()> {
//         track_any_err!(write!(self.inner.borrow_mut(), "{}", s))?;
//         Ok(())
//     }

//     pub fn write_table(&mut self, table: &Table) -> Result<()> {
//         let mut widthes = table
//             .headers
//             .iter()
//             .map(|h| h.name.len())
//             .collect::<Vec<_>>();

//         for col in 0..table.headers.len() {
//             for row in &table.rows {
//                 if let Some(item) = row.items.get(col) {
//                     widthes[col] = cmp::max(widthes[col], item.len());
//                 }
//             }
//         }

//         track!(self.write("|"))?;
//         for (h, w) in table.headers.iter().zip(widthes.iter().cloned()) {
//             let s = match h.align {
//                 Align::Left => format!(" {:<width$} |", h.name, width = w),
//                 Align::Center => format!(" {:^width$} |", h.name, width = w),
//                 Align::Right => format!(" {:>width$} |", h.name, width = w),
//             };
//             track!(self.write(&s))?;
//         }
//         track!(self.newline())?;

//         track!(self.write("|"))?;
//         for (h, w) in table.headers.iter().zip(widthes.iter().cloned()) {
//             let s = match h.align {
//                 Align::Left => format!(":{:-<width$}-|", "-", width = w),
//                 Align::Center => format!(":{:-^width$}:|", "-", width = w),
//                 Align::Right => format!("-{:->width$}:|", "-", width = w),
//             };
//             track!(self.write(&s))?;
//         }
//         track!(self.newline())?;

//         for row in &table.rows {
//             track!(self.write("|"))?;
//             for (h, (item, w)) in table
//                 .headers
//                 .iter()
//                 .zip(row.items.iter().zip(widthes.iter().cloned()))
//             {
//                 let s = match h.align {
//                     Align::Left => format!(" {:<width$} |", item, width = w),
//                     Align::Center => format!(" {:^width$} |", item, width = w),
//                     Align::Right => format!(" {:>width$} |", item, width = w),
//                 };
//                 track!(self.write(&s))?;
//             }
//             track!(self.newline())?;
//         }

//         Ok(())
//     }
// }

// #[derive(Debug)]
// pub struct Table {
//     headers: Vec<ColumnHeader>,
//     rows: Vec<Row>,
// }
// impl Table {
//     pub fn new<I>(headers: I) -> Self
//     where
//         I: Iterator<Item = ColumnHeader>,
//     {
//         let headers = headers.collect();
//         Self {
//             headers,
//             rows: Vec::new(),
//         }
//     }

//     pub fn row(&mut self) -> &mut Row {
//         self.rows.push(Row::default());
//         self.rows.last_mut().unwrap_or_else(|| unreachable!())
//     }
// }

// #[derive(Debug)]
// pub struct ColumnHeader {
//     name: String,
//     align: Align,
// }
// impl ColumnHeader {
//     pub fn new(name: &str, align: Align) -> Self {
//         Self {
//             name: name.to_owned(),
//             align,
//         }
//     }
// }

// #[derive(Debug, Default)]
// pub struct Row {
//     items: Vec<String>,
// }
// impl Row {
//     pub fn item<T>(&mut self, item: T) -> &mut Self
//     where
//         T: ToString,
//     {
//         self.items.push(item.to_string());
//         self
//     }
// }

// #[derive(Debug)]
// pub enum Align {
//     Left,
//     Center,
//     Right,
// }
