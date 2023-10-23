/// Construct a table which can be pretty printed.
///
/// Formats the contents as:
/// ```
///
/// ```
pub struct Table<const COLUMNS: usize> {
    headers: [String; COLUMNS],
    data: Vec<[String; COLUMNS]>,
}

impl<const COLUMNS: usize> Table<COLUMNS> {
    pub fn new(headers: [String; COLUMNS], data: Vec<[String; COLUMNS]>) -> Self {
        Self { headers, data }
    }
}

impl<const COLUMNS: usize> std::fmt::Display for Table<COLUMNS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut maxs = [0usize; COLUMNS];

        for (i, v) in self.headers.iter().enumerate() {
            maxs[i] = maxs[i].max(v.len());
        }

        for row in &self.data {
            for (i, v) in row.iter().enumerate() {
                maxs[i] = maxs[i].max(v.len());
            }
        }

        let mut total = 0;
        for (v, max) in self.headers.iter().zip(maxs) {
            let diff = max.saturating_sub(v.len());
            v.fmt(f)?;
            if diff > 0 {
                " ".repeat(diff as usize).fmt(f)?;
            }
            " | ".fmt(f)?;
            total += max + 3;
        }

        writeln!(f)?;
        writeln!(f, "{}", "-".repeat(total))?;

        for row in &self.data {
            for (v, max) in row.into_iter().zip(maxs) {
                let diff = max.saturating_sub(v.len());
                v.fmt(f)?;
                if diff > 0 {
                    " ".repeat(diff as usize).fmt(f)?;
                }
                " | ".fmt(f)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
