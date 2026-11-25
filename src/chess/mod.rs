//! Chess visualization module for dotmax.
//!
//! Provides tools to render chess boards from PGN or FEN using braille.

pub mod board;
pub mod sprites;

use std::io::Cursor;
use std::ops::ControlFlow;

use crate::error::DotmaxError;
pub use board::{BoardColorScheme, RenderOptions};
use pgn_reader::{Reader, SanPlus, Visitor};
use shakmaty::{Chess, Position};

/// Render a PGN string to a braille-encoded string.
///
/// This is an agnostic output suitable for terminals or web frontends.
///
/// # Errors
///
/// Returns [`DotmaxError::TerminalBackend`] if the PGN contains no readable game,
/// or a [`DotmaxError`] from board rendering (invalid grid dimensions).
///
/// # Examples
///
/// ```
/// let board = dotmax::chess::render_pgn("1. e4 *", Some(0)).unwrap();
/// assert!(board.lines().count() > 4);
/// ```
pub fn render_pgn(pgn: &str, move_index: Option<usize>) -> Result<String, DotmaxError> {
    render_pgn_with_options(pgn, move_index, &RenderOptions::default())
}

/// Render a PGN string to a braille-encoded string with options.
///
/// # Errors
///
/// Returns [`DotmaxError::TerminalBackend`] if the PGN contains no readable game,
/// or a [`DotmaxError`] from board rendering (invalid grid dimensions).
///
/// # Examples
///
/// ```
/// use dotmax::chess::{render_pgn_with_options, RenderOptions};
/// let board = render_pgn_with_options("1. e4 *", Some(0), &RenderOptions::default()).unwrap();
/// assert!(!board.is_empty());
/// ```
pub fn render_pgn_with_options(
    pgn: &str,
    move_index: Option<usize>,
    options: &RenderOptions,
) -> Result<String, DotmaxError> {
    let mut visitor = ChessVisitor::new(move_index);
    let mut reader = Reader::new(Cursor::new(pgn.as_bytes()));

    if let Ok(Some(pos)) = reader.read_game(&mut visitor) {
        let grid = board::render_position_with_options(&pos, options)?;
        let mut out = String::new();
        for row in grid.to_unicode_grid() {
            for ch in row {
                out.push(ch);
            }
            out.push('\n');
        }
        Ok(out)
    } else {
        Err(DotmaxError::TerminalBackend(
            "No game found or PGN error".to_string(),
        ))
    }
}

struct ChessVisitor {
    target_move: Option<usize>,
}

impl ChessVisitor {
    fn new(target_move: Option<usize>) -> Self {
        Self { target_move }
    }
}

/// Mainline position while a game is being visited.
///
/// `(position, plies played, stop before the next ply)`.
type Mainline = (Chess, usize, bool);

impl Visitor for ChessVisitor {
    type Tags = ();
    type Movetext = Mainline;
    type Output = Chess;

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue((Chess::default(), 0, false))
    }

    fn san(
        &mut self,
        movetext: &mut Self::Movetext,
        san_plus: SanPlus,
    ) -> ControlFlow<Self::Output> {
        let (pos, current_move, finished) = movetext;
        if *finished {
            return ControlFlow::Continue(());
        }

        if let Some(target) = self.target_move {
            if *current_move >= target {
                *finished = true;
                return ControlFlow::Continue(());
            }
        }

        if let Ok(mv) = san_plus.san.to_move(pos) {
            pos.play_unchecked(mv);
            *current_move += 1;
        } else {
            *finished = true;
        }
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, movetext: Self::Movetext) -> Self::Output {
        movetext.0
    }
}
