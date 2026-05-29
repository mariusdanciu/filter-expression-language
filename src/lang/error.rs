use winnow::error::{AddContext, ModalError, ParserError, StrContext};
use winnow::stream::Stateful;
use crate::lang::ast::ParserContext;

#[derive(Debug, Clone)]
pub struct FurthestError {
    pub remaining_len: usize,
    pub original: String,
    pub context: Vec<StrContext>,
    pub is_cut: bool,
}

impl<'s> ParserError<Stateful<&'s str, &'s ParserContext>> for FurthestError {
    type Inner = Self;

    fn from_input(input: &Stateful<&'s str, &'s ParserContext>) -> Self {
        FurthestError {
            remaining_len: input.input.len(),
            original: input.state.original_input.clone(),
            context: Vec::new(),
            is_cut: false,
        }
    }

    fn append(
        self,
        _input: &Stateful<&'s str, &'s ParserContext>,
        _token_start: &<Stateful<&'s str, &'s ParserContext> as winnow::stream::Stream>::Checkpoint,
    ) -> Self {
        self
    }

    fn or(self, other: Self) -> Self {
        // If either is cut, don't backtrack
        if self.is_cut {
            return self;
        }
        if other.is_cut {
            return other;
        }

        // Otherwise pick the furthest error
        if other.remaining_len < self.remaining_len {
            other
        } else {
            self
        }
    }

    fn into_inner(self) -> std::result::Result<Self::Inner, Self> {
        Ok(self)
    }
}

impl ModalError for FurthestError {
    fn cut(mut self) -> Self {
        self.is_cut = true;
        self
    }

    fn backtrack(mut self) -> Self {
        self.is_cut = false;
        self
    }
}

impl<'s> AddContext<Stateful<&'s str, &'s ParserContext>, StrContext> for FurthestError {
    fn add_context(
        mut self,
        _input: &Stateful<&'s str, &'s ParserContext>,
        _token_start: &<Stateful<&'s str, &'s ParserContext> as winnow::stream::Stream>::Checkpoint,
        context: StrContext,
    ) -> Self {
        self.context.push(context);
        self
    }
}

impl std::fmt::Display for FurthestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_pos = self.original.len() - self.remaining_len;

        writeln!(f, "Parse error at position {}:", error_pos)?;
        writeln!(f, "{}", self.original)?;
        writeln!(f, "{}^", " ".repeat(error_pos))?;

        if !self.context.is_empty() {
            writeln!(f, "\nContext:")?;
            for ctx in &self.context {
                writeln!(f, "  {:?}", ctx)?;
            }
        }

        Ok(())
    }
}
