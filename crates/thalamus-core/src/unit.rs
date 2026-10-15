//! UCUM units, case-sensitive form (`c/s`).
//!
//! `thalamus.definition.Unit` holds only the character set of a unit, because ESS has no grammar-typed
//! string (`UNMAPPED:` in `spec/domains/definition.yaml`). [`parse_ucum`] holds the grammar: the
//! unity `1` and other integer factors, annotations `{…}`, metric prefixes on metric atoms, integer
//! exponents, `.` products, `/` quotients, parenthesised terms and the `10*` and `10^` atoms. Atoms
//! and prefixes are checked against [`ATOMS`] and [`PREFIXES`], the case-sensitive codes of UCUM.
//!
//! Pure: no I/O and no clock.

use std::fmt;

/// A text that parsed as a case-sensitive UCUM unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UcumUnit {
    text: String,
}

impl UcumUnit {
    /// The unit as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for UcumUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Why a text is not a UCUM unit, and the byte offset where that was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitError {
    /// Byte offset into the text.
    pub at: usize,
    /// What was wrong there.
    pub kind: UnitErrorKind,
}

/// What made a text not a UCUM unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitErrorKind {
    /// The text is empty; the unity is written `1`.
    Empty,
    /// A character the grammar does not admit at this point.
    UnexpectedChar(char),
    /// The text ends where a component is required.
    UnexpectedEnd,
    /// An annotation `{` with no closing `}`.
    UnclosedAnnotation,
    /// A `[` with no closing `]`.
    UnclosedBracket,
    /// A `(` with no closing `)`.
    UnclosedParenthesis,
    /// A symbol that is neither a UCUM atom nor a prefix on a metric atom.
    UnknownSymbol(String),
    /// A `(` nested deeper than [`MAX_DEPTH`] levels.
    TooDeep,
}

/// The deepest nesting of parentheses [`parse_ucum`] admits. UCUM sets no limit; this one keeps
/// the recursive descent off the end of the stack.
pub const MAX_DEPTH: usize = 64;

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a UCUM unit at byte {}: ", self.at)?;
        match &self.kind {
            UnitErrorKind::Empty => f.write_str("empty (the unity is `1`)"),
            UnitErrorKind::UnexpectedChar(c) => write!(f, "unexpected {c:?}"),
            UnitErrorKind::UnexpectedEnd => f.write_str("a component is missing at the end"),
            UnitErrorKind::UnclosedAnnotation => f.write_str("annotation `{` is not closed"),
            UnitErrorKind::UnclosedBracket => f.write_str("`[` is not closed"),
            UnitErrorKind::UnclosedParenthesis => f.write_str("`(` is not closed"),
            UnitErrorKind::UnknownSymbol(symbol) => write!(
                f,
                "`{symbol}` is neither a UCUM atom nor a prefix on a metric atom"
            ),
            UnitErrorKind::TooDeep => {
                write!(f, "parentheses nest deeper than {MAX_DEPTH} levels")
            }
        }
    }
}

impl std::error::Error for UnitError {}

/// Parses `text` as a case-sensitive UCUM unit.
///
/// # Errors
///
/// [`UnitError`] naming the first byte offset where `text` leaves the UCUM grammar or names a
/// symbol that is not a UCUM atom.
pub fn parse_ucum(text: &str) -> Result<UcumUnit, UnitError> {
    if text.is_empty() {
        return Err(UnitError {
            at: 0,
            kind: UnitErrorKind::Empty,
        });
    }
    let mut parser = Parser {
        text,
        pos: 0,
        depth: 0,
    };
    parser.main_term()?;
    Ok(UcumUnit {
        text: text.to_owned(),
    })
}

/// The case-sensitive UCUM prefixes. Two-character codes come first so `da` is tried before `d`.
pub const PREFIXES: &[&str] = &[
    "da", "Ki", "Mi", "Gi", "Ti", "Y", "Z", "E", "P", "T", "G", "M", "k", "h", "d", "c", "m", "u",
    "n", "p", "f", "a", "z", "y",
];

/// The case-sensitive UCUM atoms, each with whether it is metric (takes a prefix).
pub const ATOMS: &[(&str, bool)] = &[
    // Base units.
    ("m", true),
    ("s", true),
    ("g", true),
    ("rad", true),
    ("K", true),
    ("C", true),
    ("cd", true),
    // Dimensionless.
    ("10*", false),
    ("10^", false),
    ("[pi]", false),
    ("%", false),
    ("[ppth]", false),
    ("[ppm]", false),
    ("[ppb]", false),
    ("[pptr]", false),
    // SI.
    ("mol", true),
    ("sr", true),
    ("Hz", true),
    ("N", true),
    ("Pa", true),
    ("J", true),
    ("W", true),
    ("A", true),
    ("V", true),
    ("F", true),
    ("Ohm", true),
    ("S", true),
    ("Wb", true),
    ("Cel", true),
    ("T", true),
    ("H", true),
    ("lm", true),
    ("lx", true),
    ("Bq", true),
    ("Gy", true),
    ("Sv", true),
    // Other units from ISO 1000, ISO 2955 and ANSI X3.50.
    ("gon", false),
    ("deg", false),
    ("'", false),
    ("''", false),
    ("l", true),
    ("L", true),
    ("ar", true),
    ("min", false),
    ("h", false),
    ("d", false),
    ("a_t", false),
    ("a_j", false),
    ("a_g", false),
    ("a", false),
    ("wk", false),
    ("mo_s", false),
    ("mo_j", false),
    ("mo_g", false),
    ("mo", false),
    ("t", true),
    ("bar", true),
    ("atm", false),
    ("u", true),
    ("eV", true),
    ("AU", false),
    ("pc", true),
    // Natural units.
    ("[c]", true),
    ("[h]", true),
    ("[k]", true),
    ("[eps_0]", true),
    ("[mu_0]", true),
    ("[e]", true),
    ("[m_e]", true),
    ("[m_p]", true),
    ("[G]", true),
    ("[g]", true),
    ("[ly]", true),
    ("gf", true),
    // CGS.
    ("Ky", true),
    ("Gal", true),
    ("dyn", true),
    ("erg", true),
    ("P", true),
    ("Bi", true),
    ("St", true),
    ("Mx", true),
    ("G", true),
    ("Oe", true),
    ("Gb", true),
    ("sb", true),
    ("Lmb", true),
    ("ph", true),
    ("Ci", true),
    ("R", true),
    ("RAD", true),
    ("REM", true),
    // International customary.
    ("[in_i]", false),
    ("[ft_i]", false),
    ("[yd_i]", false),
    ("[mi_i]", false),
    ("[fth_i]", false),
    ("[nmi_i]", false),
    ("[kn_i]", false),
    ("[sin_i]", false),
    ("[sft_i]", false),
    ("[syd_i]", false),
    ("[cin_i]", false),
    ("[cft_i]", false),
    ("[cyd_i]", false),
    ("[bf_i]", false),
    ("[cr_i]", false),
    ("[mil_i]", false),
    ("[cml_i]", false),
    ("[hd_i]", false),
    // US survey lengths.
    ("[ft_us]", false),
    ("[yd_us]", false),
    ("[in_us]", false),
    ("[rd_us]", false),
    ("[ch_us]", false),
    ("[lk_us]", false),
    ("[rch_us]", false),
    ("[rlk_us]", false),
    ("[fth_us]", false),
    ("[fur_us]", false),
    ("[mi_us]", false),
    ("[acr_us]", false),
    ("[srd_us]", false),
    ("[smi_us]", false),
    ("[sct]", false),
    ("[twp]", false),
    ("[mil_us]", false),
    // British imperial lengths.
    ("[in_br]", false),
    ("[ft_br]", false),
    ("[rd_br]", false),
    ("[ch_br]", false),
    ("[lk_br]", false),
    ("[fth_br]", false),
    ("[pc_br]", false),
    ("[yd_br]", false),
    ("[mi_br]", false),
    ("[nmi_br]", false),
    ("[kn_br]", false),
    ("[acr_br]", false),
    // US volumes.
    ("[gal_us]", false),
    ("[bbl_us]", false),
    ("[qt_us]", false),
    ("[pt_us]", false),
    ("[gil_us]", false),
    ("[foz_us]", false),
    ("[fdr_us]", false),
    ("[min_us]", false),
    ("[crd_us]", false),
    ("[bu_us]", false),
    ("[gal_wi]", false),
    ("[pk_us]", false),
    ("[dqt_us]", false),
    ("[dpt_us]", false),
    ("[tbs_us]", false),
    ("[tsp_us]", false),
    ("[cup_us]", false),
    ("[foz_m]", false),
    ("[cup_m]", false),
    ("[tsp_m]", false),
    ("[tbs_m]", false),
    // British imperial volumes.
    ("[gal_br]", false),
    ("[pk_br]", false),
    ("[bu_br]", false),
    ("[qt_br]", false),
    ("[pt_br]", false),
    ("[gil_br]", false),
    ("[foz_br]", false),
    ("[fdr_br]", false),
    ("[min_br]", false),
    // Avoirdupois, troy and apothecaries' weights.
    ("[gr]", false),
    ("[lb_av]", false),
    ("[oz_av]", false),
    ("[dr_av]", false),
    ("[scwt_av]", false),
    ("[lcwt_av]", false),
    ("[ston_av]", false),
    ("[lton_av]", false),
    ("[stone_av]", false),
    ("[lbf_av]", false),
    ("[pwt_tr]", false),
    ("[oz_tr]", false),
    ("[lb_tr]", false),
    ("[sc_ap]", false),
    ("[dr_ap]", false),
    ("[oz_ap]", false),
    ("[lb_ap]", false),
    ("[oz_m]", false),
    // Typesetting.
    ("[lne]", false),
    ("[pnt]", false),
    ("[pca]", false),
    ("[pnt_pr]", false),
    ("[pca_pr]", false),
    ("[pied]", false),
    ("[pouce]", false),
    ("[ligne]", false),
    ("[didot]", false),
    ("[cicero]", false),
    // Legacy heat and temperature.
    ("[degF]", false),
    ("[degR]", false),
    ("[degRe]", false),
    ("cal_[15]", true),
    ("cal_[20]", true),
    ("cal_m", true),
    ("cal_IT", true),
    ("cal_th", true),
    ("cal", true),
    ("[Cal]", false),
    ("[Btu_39]", false),
    ("[Btu_59]", false),
    ("[Btu_60]", false),
    ("[Btu_m]", false),
    ("[Btu_IT]", false),
    ("[Btu_th]", false),
    ("[Btu]", false),
    ("[HP]", false),
    ("tex", true),
    ("[den]", false),
    // Clinical and chemical.
    ("m[H2O]", true),
    ("m[Hg]", true),
    ("[in_i'H2O]", false),
    ("[in_i'Hg]", false),
    ("[PRU]", false),
    ("[wood'U]", false),
    ("[diop]", false),
    ("[p'diop]", false),
    ("%[slope]", false),
    ("[mesh_i]", false),
    ("[Ch]", false),
    ("[drp]", false),
    ("[hnsf'U]", false),
    ("[MET]", false),
    ("[hp'_X]", false),
    ("[hp'_C]", false),
    ("[hp'_M]", false),
    ("[hp'_Q]", false),
    ("[hp_X]", false),
    ("[hp_C]", false),
    ("[hp_M]", false),
    ("[hp_Q]", false),
    ("[kp_X]", false),
    ("[kp_C]", false),
    ("[kp_M]", false),
    ("[kp_Q]", false),
    ("eq", true),
    ("osm", true),
    ("[pH]", false),
    ("g%", true),
    ("[S]", false),
    ("[HPF]", false),
    ("[LPF]", false),
    ("kat", true),
    ("U", true),
    ("[iU]", true),
    ("[IU]", true),
    ("[arb'U]", false),
    ("[USP'U]", false),
    ("[GPL'U]", false),
    ("[MPL'U]", false),
    ("[APL'U]", false),
    ("[beth'U]", false),
    ("[anti'Xa'U]", false),
    ("[todd'U]", false),
    ("[dye'U]", false),
    ("[smgy'U]", false),
    ("[bdsk'U]", false),
    ("[ka'U]", false),
    ("[knk'U]", false),
    ("[mclg'U]", false),
    ("[tb'U]", false),
    ("[CCID_50]", false),
    ("[TCID_50]", false),
    ("[EID_50]", false),
    ("[PFU]", false),
    ("[FFU]", false),
    ("[CFU]", false),
    ("[IR]", false),
    ("[BAU]", false),
    ("[AU]", false),
    ("[Amb'a'1'U]", false),
    ("[PNU]", false),
    ("[Lf]", false),
    ("[D'ag'U]", false),
    ("[FEU]", false),
    ("[ELU]", false),
    ("[EU]", false),
    // Levels.
    ("Np", true),
    ("B", true),
    ("B[SPL]", true),
    ("B[V]", true),
    ("B[mV]", true),
    ("B[uV]", true),
    ("B[10.nV]", true),
    ("B[W]", true),
    ("B[kW]", true),
    // Miscellaneous.
    ("st", true),
    ("Ao", false),
    ("b", false),
    ("att", false),
    ("mho", true),
    ("[psi]", false),
    ("circ", false),
    ("sph", false),
    ("[car_m]", false),
    ("[car_Au]", false),
    ("[smoot]", false),
    ("[m/s2/Hz^(1/2)]", false),
    ("[NTU]", false),
    ("[FNU]", false),
    // Information technology.
    ("bit_s", false),
    ("bit", true),
    ("By", true),
    ("Bd", true),
];

fn atom(symbol: &str) -> Option<bool> {
    ATOMS
        .iter()
        .find(|(code, _)| *code == symbol)
        .map(|(_, metric)| *metric)
}

/// Whether `symbol` is an atom, or a prefix followed by a metric atom.
fn known(symbol: &str) -> bool {
    atom(symbol).is_some()
        || PREFIXES.iter().any(|prefix| {
            symbol
                .strip_prefix(prefix)
                .is_some_and(|rest| atom(rest) == Some(true))
        })
}

/// A character that may stand in a symbol outside square brackets.
fn symbol_char(byte: u8) -> bool {
    (33..=126).contains(&byte)
        && !matches!(byte, b'.' | b'/' | b'(' | b')' | b'{' | b'}' | b'[' | b']')
}

/// A character that may stand inside the square brackets of an atom.
fn bracket_char(byte: u8) -> bool {
    (33..=126).contains(&byte) && !matches!(byte, b'{' | b'}' | b'[' | b']')
}

/// A character that may stand inside an annotation: UCUM § 6 admits every character 33–126 and
/// refuses only nested curly braces.
fn annotation_char(byte: u8) -> bool {
    (33..=126).contains(&byte) && !matches!(byte, b'{' | b'}')
}

/// Recursive descent over the UCUM grammar:
///
/// ```text
/// main-term   := "/" term | term
/// term        := component (("." | "/") component)*
/// component   := "(" term ")" | annotation | factor | simple-unit exponent? annotation?
/// simple-unit := ATOM | PREFIX ATOM[metric]
/// exponent    := ("+" | "-")? digits
/// annotation  := "{" printable ASCII except braces "}"
/// ```
struct Parser<'a> {
    text: &'a str,
    pos: usize,
    /// How many `(` enclose the current position.
    depth: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn unexpected(&self) -> UnitError {
        match self.text[self.pos..].chars().next() {
            Some(c) => fail(self.pos, UnitErrorKind::UnexpectedChar(c)),
            None => fail(self.pos, UnitErrorKind::UnexpectedEnd),
        }
    }

    fn main_term(&mut self) -> Result<(), UnitError> {
        if self.peek() == Some(b'/') {
            self.pos += 1;
        }
        self.term()?;
        if self.pos == self.text.len() {
            Ok(())
        } else {
            Err(self.unexpected())
        }
    }

    fn term(&mut self) -> Result<(), UnitError> {
        self.component()?;
        while let Some(b'.' | b'/') = self.peek() {
            self.pos += 1;
            self.component()?;
        }
        Ok(())
    }

    fn component(&mut self) -> Result<(), UnitError> {
        match self.peek() {
            Some(b'(') => {
                let open = self.pos;
                if self.depth == MAX_DEPTH {
                    return Err(fail(open, UnitErrorKind::TooDeep));
                }
                self.depth += 1;
                self.pos += 1;
                self.term()?;
                self.depth -= 1;
                match self.peek() {
                    Some(b')') => {
                        self.pos += 1;
                        Ok(())
                    }
                    None => Err(fail(open, UnitErrorKind::UnclosedParenthesis)),
                    Some(_) => Err(self.unexpected()),
                }
            }
            Some(b'{') => self.annotation(),
            Some(b'[') => self.simple_unit(),
            Some(byte) if symbol_char(byte) => self.simple_unit(),
            _ => Err(self.unexpected()),
        }
    }

    fn annotation(&mut self) -> Result<(), UnitError> {
        let open = self.pos;
        self.pos += 1;
        loop {
            match self.peek() {
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(());
                }
                Some(byte) if annotation_char(byte) => self.pos += 1,
                None => return Err(fail(open, UnitErrorKind::UnclosedAnnotation)),
                Some(_) => return Err(self.unexpected()),
            }
        }
    }

    /// A factor, or a simple unit with its exponent, followed by an optional annotation.
    fn simple_unit(&mut self) -> Result<(), UnitError> {
        let start = self.pos;
        loop {
            match self.peek() {
                Some(b'[') => {
                    let open = self.pos;
                    self.pos += 1;
                    loop {
                        match self.peek() {
                            Some(b']') => {
                                self.pos += 1;
                                break;
                            }
                            Some(byte) if bracket_char(byte) => self.pos += 1,
                            None => return Err(fail(open, UnitErrorKind::UnclosedBracket)),
                            Some(_) => return Err(self.unexpected()),
                        }
                    }
                }
                Some(byte) if symbol_char(byte) => self.pos += 1,
                _ => break,
            }
        }
        let symbol = &self.text[start..self.pos];
        if symbol.bytes().all(|byte| byte.is_ascii_digit()) {
            // A factor takes neither an exponent nor an annotation.
            return if self.peek() == Some(b'{') {
                Err(self.unexpected())
            } else {
                Ok(())
            };
        }
        let stem = stem(symbol);
        if !known(stem) {
            return Err(fail(start, UnitErrorKind::UnknownSymbol(stem.to_owned())));
        }
        if self.peek() == Some(b'{') {
            self.annotation()?;
        }
        Ok(())
    }
}

fn fail(at: usize, kind: UnitErrorKind) -> UnitError {
    UnitError { at, kind }
}

/// `symbol` without its trailing exponent: digits, optionally signed, after the last non-digit.
fn stem(symbol: &str) -> &str {
    let bytes = symbol.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1].is_ascii_digit() {
        end -= 1;
    }
    if end < bytes.len() && end > 0 && matches!(bytes[end - 1], b'+' | b'-') {
        end -= 1;
    }
    &symbol[..end]
}
