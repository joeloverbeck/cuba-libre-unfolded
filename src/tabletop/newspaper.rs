//! The newspaper (#208): selected B1 · El Correo de la Isla, a front page over a scrim that
//! prints one action report, and the pile of the current card's editions beside the
//! Propaganda pips. Which report is printed, and when, is `crate::editions`'s.
//!
//! Layout authority (foundation 4): `art/atlas/atlas-newspaper.figcraft` owns the
//! structure, places and sizes; [`fig`] copies its frames' numbers, which
//! `atlas-newspaper.layout.json` records for the native comparison (NEWSPAPER.md). The
//! masthead, the dice and the pile are VectorCraft exports and the plates FigCraft's
//! (`newspaper.sha256`).
//!
//! The page prints the report's own words and nothing else. Its lines are broken once, in
//! the layout's own units ([`Setting`]), so an edition has the same lines, columns and
//! pages in every window.
use super::palette::{INK, PAPER, QUIET};
use super::*;
use crate::{
    editions::{Edition, Open},
    reports::{Action as Did, Article, FromTo, Report, Row},
};
use bevy::text::{
    ComputedTextBlock, FontCx, Justify, LayoutCx, LetterSpacing, LineBreak, LineHeight, RemSize,
    TextBounds, TextElement, TextPipeline,
};
use bevy::ui::widget::NodeImageMode;
use serde_json::json;

/// Node coordinates of `art/atlas/atlas-newspaper.figcraft`'s 16:9 frames, in their units:
/// the owner's display. Everything on the page is measured from the page's own corner.
mod fig {
    pub const FRAME: [f32; 2] = [3840., 2160.];
    /// Where the scrim starts: the top band stays clear.
    pub const SCRIM_TOP: f32 = 206.;
    /// The page's left, top and width.
    pub const PAGE: [f32; 3] = [1140., 218., 1560.];
    /// How tall a page may be: to 42 from the window's foot.
    pub const TALLEST: f32 = 1900.;
    /// How far the page's shadow reaches past it: left, top, right, bottom.
    pub const SHADOW: [f32; 4] = [42., 32., 42., 52.];
    pub const PAD: f32 = 40.;
    /// The width between the page's margins, which the rules fill.
    pub const FULL: f32 = 1480.;
    pub const MASTHEAD: [f32; 4] = [40., 24., 1480., 200.];
    pub const RULE: f32 = 2.;
    /// The folio line: left, top, width.
    pub const FOLIO: [f32; 3] = [40., 236., 700.];
    pub const ABOUT: [f32; 3] = [740., 236., 780.];
    pub const FOLIO_TYPE: [f32; 2] = [25., 30.];
    pub const FOLIO_RULE: f32 = 276.;
    /// A row of tabs under the folio line: its top, a tab's height, its words' inset and
    /// the gaps between tabs and between rows. One row moves what is below down by `BAND`.
    pub const TABS: f32 = 274.;
    pub const TAB: f32 = 60.;
    pub const TAB_PAD: f32 = 30.;
    pub const TAB_GAP: f32 = 10.;
    pub const TAB_ROW: f32 = 68.;
    pub const BAND: f32 = 74.;
    pub const HEADLINE: f32 = 296.;
    pub const HEADLINE_TYPE: [f32; 2] = [84., 92.];
    /// Below the headline: above the summary, whose left, width and type follow, and above
    /// the rule that closes them.
    pub const ABOVE_SUMMARY: f32 = 14.;
    pub const SUMMARY: [f32; 2] = [100., 1360.];
    pub const SUMMARY_TYPE: [f32; 2] = [34., 42.];
    pub const ABOVE_RULE: f32 = 22.;
    /// From a rule to the columns below it.
    pub const BELOW_RULE: f32 = 24.;
    pub const COLUMNS: [f32; 3] = [40., 543., 1046.];
    pub const COLUMN: f32 = 453.;
    pub const COLUMN_RULES: [f32; 2] = [516., 1019.];
    pub const TITLE_TYPE: [f32; 2] = [38., 44.];
    pub const TEXT_TYPE: [f32; 2] = [27., 36.];
    /// Between two blocks of a column.
    pub const BETWEEN: f32 = 26.;
    /// From a title to its words: an article's, which run on, and a note's, whose
    /// sentences stand apart.
    pub const BELOW_TITLE: [f32; 2] = [6., 8.];
    pub const NOTE: f32 = 450.;
    pub const BETWEEN_SENTENCES: f32 = 14.;
    /// The die at the top right of its article, whose words keep clear of it.
    pub const DIE: [f32; 2] = [381., 72.];
    pub const BESIDE_DIE: f32 = 369.;
    pub const DIE_HEIGHT: f32 = 80.;
    /// The standings at the top of the third column: its width, the headings' top, and the
    /// rows' first rule and height.
    pub const TABLE: f32 = 474.;
    pub const HEADINGS: f32 = 58.;
    pub const HEADING_TYPE: [f32; 2] = [20., 24.];
    pub const ROWS: f32 = 90.;
    pub const ROW: f32 = 52.;
    pub const EMBLEM: [f32; 3] = [0., 9., 34.];
    /// A row's words: their top, then the name's and each figure's left and width.
    pub const ROW_TEXT: f32 = 11.;
    pub const NAME: [f32; 2] = [44., 140.];
    pub const RESOURCES: [f32; 2] = [180., 132.];
    pub const MARGIN: [f32; 2] = [322., 151.];
    pub const ROW_TYPE: [f32; 2] = [25., 30.];
    /// The table's height, and what the first block under it stands below.
    pub const STANDINGS: f32 = 300.;
    pub const BELOW_STANDINGS: f32 = 28.;
    /// From the lowest block to the column rules' end, then to the foot line, then to the
    /// page's own foot.
    pub const BELOW_COLUMNS: f32 = 24.;
    pub const ABOVE_FOOT: f32 = 20.;
    pub const FOOT: f32 = 50.;
    pub const FOOT_TYPE: [f32; 2] = [22., 26.];
    /// The foot line's three texts: left and width.
    pub const SOURCE: [f32; 2] = [40., 700.];
    pub const PAGES: [f32; 2] = [480., 600.];
    pub const AWAY: [f32; 2] = [780., 740.];
    /// The pile's parts, from its area's corner.
    pub const PILE: [f32; 4] = [0., 0., 300., 190.];
    pub const DISC: [f32; 4] = [252., 120., 64., 64.];
    pub const COUNT: [f32; 3] = [252., 134., 64.];
    pub const COUNT_TYPE: [f32; 2] = [32., 36.];
    pub const CAPTION: [f32; 2] = [0., 200.];
    pub const CAPTION_TYPE: [f32; 2] = [28., 34.];
}

/// The control that puts the open edition away: the whole window.
pub(super) const NAME: &str = "edition";
/// The pile's control, and the main view's area it lies in.
pub(super) const PILE: &str = "editions";
pub(super) const AREA: &str = PILE;
/// Over the transfer modal and the focus mark.
const LAYER: i32 = transfer::LAYER + 10;

const PAGE_PLATE: &str = "atlas-newspaper-page.png";
/// The plates' slices in frame units: the page's, clear of its shadow, and a tab's. The
/// exports are 2 px per unit.
const PAGE_SLICE: f32 = 90.;
const TAB_SLICE: f32 = 8.;
const TAB_PLATES: [&str; 2] = ["atlas-newspaper-tab.png", "atlas-newspaper-tab-open.png"];
/// The accepted ink rule and the hairline the cards' briefings use.
const RULE_INK: &str = "atlas-popover-rule-ink.png";
const RULE: &str = "atlas-briefing-rule.png";

const SOURCE: &str = "Written from the committed result";
const AWAY: &str = "Click, Escape, Enter or Space puts it away";
const STANDINGS: &str = "The standings";

/// What the human pressed on the newspaper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Press {
    PutAway,
    Reopen,
    Show(usize),
}

pub(super) fn press(ui: &mut Interface, press: Press) {
    match press {
        Press::PutAway => ui.editions.put_away(),
        // The pile shows over the scrim, and a click anywhere puts the page away.
        Press::Reopen if ui.editions.open().is_some() => ui.editions.put_away(),
        Press::Reopen => ui.editions.reopen(),
        Press::Show(number) => ui.editions.show(number),
    }
    ui.dirty = true;
}

/// The stop Tab reaches while the pile lies on the table.
pub(super) fn focusable(ui: &Interface) -> Option<(String, Action)> {
    (ui.editions.pile() > 0).then(|| (PILE.into(), Action::Edition(Press::Reopen)))
}

/// Opens an edition that is due, once the pieces and the die have come to rest: nothing
/// moves, and a landed die has shown its result. The human's looking about, a hover's
/// dwell or the camera, does not delay it.
pub(super) fn open(world: &mut World) {
    let ui = world.resource::<Interface>();
    if !ui.editions.due() || ui.editions.open().is_some() {
        return;
    }
    if !settled::resting(world).is_empty() {
        return;
    }
    let mut ui = world.resource_mut::<Interface>();
    ui.editions.open_next();
    ui.dirty = true;
}

/// Whether an edition has still to open or to be drawn, which the development seat waits
/// for.
pub(super) fn unsettled(world: &World) -> bool {
    let editions = &world.resource::<Interface>().editions;
    (editions.due() && editions.open().is_none())
        || editions.open().is_some_and(|(open, edition)| {
            world.resource::<Drawn>().page != Some((edition.number, open))
        })
}

/// The open edition takes the keyboard before anything beneath it: Escape, Enter or Space
/// puts it away, Left and Right turn its page, and Tab turns a reopened one to the card's
/// next. Nothing beneath reads a key, except the seats' own, which work at any time.
pub(super) fn keys(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut ui: ResMut<Interface>,
    drawn: Res<Drawn>,
) {
    if ui.editions.open().is_none() || keys.get_just_pressed().next().is_none() {
        return;
    }
    const AWAY: [KeyCode; 4] = [
        KeyCode::Escape,
        KeyCode::Enter,
        KeyCode::NumpadEnter,
        KeyCode::Space,
    ];
    let back = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keys.any_just_pressed(AWAY) {
        press(&mut ui, Press::PutAway);
    } else if keys.just_pressed(KeyCode::Tab) {
        ui.editions.step(back);
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        ui.editions.turn(true, drawn.pages);
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        ui.editions.turn(false, drawn.pages);
    }
    let pressed: Vec<KeyCode> = keys.get_just_pressed().copied().collect();
    for key in pressed {
        if !faction_seats::KEYS.contains(&key) {
            keys.clear_just_pressed(key);
        }
    }
}

/// A change stays on one line, and a number with the word it counts.
fn bound(text: &str) -> String {
    const NBSP: char = '\u{a0}';
    let chars: Vec<char> = text.chars().collect();
    let at = |i: usize| chars.get(i).copied();
    (0..chars.len())
        .map(|i| {
            let around = (i.checked_sub(1).and_then(at), at(i + 1));
            let arrow = around.0 == Some('→') || around.1 == Some('→');
            let counts = around.0.is_some_and(|c| c.is_ascii_digit())
                && around.1.is_some_and(|c| c.is_ascii_alphabetic());
            match chars[i] {
                ' ' if arrow || counts => NBSP,
                other => other,
            }
        })
        .collect()
}

/// The same words with their plain spaces.
fn plain(text: &str) -> String {
    text.replace(['\u{a0}', '\n'], " ")
}

/// A signed figure, with the minus sign.
fn signed(value: i16) -> String {
    value.to_string().replace('-', "−")
}

/// What a figure of the standings says, and whether it changed.
fn figure(of: FromTo) -> (String, bool) {
    if of.from == of.to {
        (signed(of.to), false)
    } else {
        (
            bound(&format!("{} → {}", signed(of.from), signed(of.to))),
            true,
        )
    }
}

/// The action's own name, as the folio line and the tabs give it.
fn action(report: &Report) -> Option<&'static str> {
    match report.action {
        Did::Undecided => None,
        Did::Pass => Some("Pass"),
        Did::Terror => Some("Terror"),
        Did::EventUnshaded | Did::EventShaded => Some("Event"),
        Did::Assault => Some("Assault"),
        Did::Construct => Some("Limited Construct"),
    }
}

/// The folio line's left: which edition of which card.
fn folio(edition: &Edition) -> String {
    let report = &edition.report;
    format!(
        "No. {}{} · Card {} · {}",
        edition.number,
        if report.complete { "" } else { " · partway" },
        report.card,
        report.card_title
    )
}

/// Its right: whose action, in which role, and the box it is recorded in. The authority
/// records no box for an Event, whose own name stands there.
fn about(report: &Report) -> String {
    let what = match (report.complete, action(report)) {
        (true, name) => report.recorded.or(name).map(str::to_owned),
        (false, name) => name.map(|name| format!("{name} in progress")),
    };
    [
        Some(report.faction.name().to_owned()),
        Some(report.role.to_owned()),
        what,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ")
}

/// A tab's words.
fn tab(edition: &Edition) -> String {
    let what = match (edition.report.complete, action(&edition.report)) {
        (false, _) => "partway".to_owned(),
        (true, Some("Pass")) => "Pass".into(),
        (true, Some(name)) => format!("{name} ends"),
        (true, None) => "ends".into(),
    };
    format!("No. {} · {what}", edition.number)
}

fn foot(page: usize, pages: usize) -> Option<String> {
    (pages > 1).then(|| format!("Page {} of {pages} · Left and Right turn it", page + 1))
}

/// The page's three faces.
#[derive(Clone, Copy)]
struct Faces<'a> {
    regular: &'a Handle<Font>,
    bold: &'a Handle<Font>,
    headline: &'a Handle<Font>,
}

impl<'a> Faces<'a> {
    fn of(art: &'a Art) -> Self {
        Self {
            regular: &art.font,
            bold: &art.bold,
            headline: &art.headline,
        }
    }
}

/// Breaks text into lines in the layout's own units, so every window has the same ones.
struct Setting<'a> {
    fonts: &'a Assets<Font>,
    font_cx: &'a mut FontCx,
    layout_cx: &'a mut LayoutCx,
}

impl Setting<'_> {
    /// How wide `text` is on one line, in `face` at `size`.
    fn width(&mut self, text: &str, face: &Handle<Font>, size: f32) -> f32 {
        let font = TextFont {
            font: face.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        };
        let mut block = ComputedTextBlock::default();
        let element = TextElement::Text {
            text,
            font: &font,
            color: Color::BLACK,
            line_height: LineHeight::Px(size),
            letter_spacing: LetterSpacing::default(),
        };
        TextPipeline::default()
            .update_buffer(
                self.fonts,
                std::iter::once((Entity::PLACEHOLDER, 0, element)),
                LineBreak::NoWrap,
                Justify::Left,
                TextBounds::UNBOUNDED,
                1.,
                &mut block,
                self.font_cx,
                self.layout_cx,
                Vec2::ONE,
                RemSize::default(),
            )
            .expect("The admitted fonts set the page's text");
        block.buffer().width()
    }

    /// `text` broken at its spaces into lines within `width`. Parley breaks at a no-break
    /// space that a line ends with, so it does not break them.
    fn lines(&mut self, text: &str, face: &Handle<Font>, size: f32, width: f32) -> Vec<String> {
        let mut lines: Vec<String> = vec![];
        for word in text.split(' ') {
            if let Some(line) = lines.last_mut() {
                let longer = format!("{line} {word}");
                if self.width(&longer, face, size) <= width {
                    *line = longer;
                    continue;
                }
            }
            lines.push(word.to_owned());
        }
        lines
    }

    /// The same number of lines, as even as they can be: set in the narrowest width that
    /// keeps it.
    fn balanced(&mut self, text: &str, face: &Handle<Font>, size: f32, width: f32) -> Vec<String> {
        let full = self.lines(text, face, size, width);
        let (mut narrow, mut wide) = (0., width);
        while full.len() > 1 && wide - narrow > 1. {
            let middle = (narrow + wide) / 2.;
            if self.lines(text, face, size, middle).len() > full.len() {
                narrow = middle;
            } else {
                wide = middle;
            }
        }
        self.lines(text, face, size, wide)
    }
}

/// One article or note, set.
struct Block {
    title: Vec<String>,
    die: Option<u8>,
    /// Each paragraph's lines: an article is one, a note has one for each sentence.
    paragraphs: Vec<Vec<String>>,
    /// The title's width, and the paragraphs'.
    width: f32,
    measure: f32,
    below_title: f32,
}

impl Block {
    fn height(&self) -> f32 {
        let lines = self.paragraphs.iter().map(Vec::len).sum::<usize>() as f32;
        let gaps = self.paragraphs.len().saturating_sub(1) as f32;
        let words = self.text_top() + lines * fig::TEXT_TYPE[1] + gaps * fig::BETWEEN_SENTENCES;
        match self.die {
            Some(_) => words.max(fig::DIE_HEIGHT),
            None => words,
        }
    }
    fn text_top(&self) -> f32 {
        self.title.len() as f32 * fig::TITLE_TYPE[1] + self.below_title
    }
}

/// Where one block stands: its place in the report, its column and its top there.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Stands {
    block: usize,
    column: usize,
    top: f32,
}

/// One page's blocks and the height of its columns.
#[derive(Clone, Debug, PartialEq)]
struct Page {
    stands: Vec<Stands>,
    height: f32,
}

/// Fills three columns of `height` with blocks of the `heights`, the first being block
/// `from`, in order. `first` says that the standings head the third column. What does not
/// fit is left; a block taller than the columns stands alone in an empty one.
fn filled(heights: &[f32], from: usize, height: f32, first: bool) -> Vec<Stands> {
    let tops = [
        0.,
        0.,
        if first {
            fig::STANDINGS + fig::BELOW_STANDINGS
        } else {
            0.
        },
    ];
    let mut stands = vec![];
    let (mut column, mut top) = (0, tops[0]);
    for (block, tall) in heights.iter().enumerate().skip(from) {
        while top + tall > height && top > 0. {
            column += 1;
            if column == tops.len() {
                return stands;
            }
            top = tops[column];
        }
        stands.push(Stands { block, column, top });
        top += tall + fig::BETWEEN;
    }
    stands
}

/// The edition's pages: each as low as holds what is left, up to `tallest` columns (the
/// first page's, then the others'), and never lower than the standings on the first.
fn pages(heights: &[f32], tallest: [f32; 2]) -> Vec<Page> {
    let mut pages: Vec<Page> = vec![];
    let mut from = 0;
    while pages.is_empty() || from < heights.len() {
        let first = pages.is_empty();
        let tallest = tallest[usize::from(!first)];
        let least = if first { fig::STANDINGS } else { 0. };
        let mut height = least;
        let mut stands = filled(heights, from, height, first);
        while from + stands.len() < heights.len() && height < tallest {
            height = (height + 2.).min(tallest);
            stands = filled(heights, from, height, first);
        }
        let end = stands
            .iter()
            .map(|s| s.top + heights[s.block])
            .fold(least, f32::max);
        from += stands.len();
        pages.push(Page {
            stands,
            height: end,
        });
    }
    pages
}

/// An edition as it is set.
struct Sheet {
    /// The tabs of a reopened one: each row's numbers, words and widths.
    tabs: Vec<Vec<(usize, String, f32)>>,
    headline: Vec<String>,
    summary: Vec<String>,
    blocks: Vec<Block>,
    pages: Vec<Page>,
}

impl Sheet {
    fn set(setting: &mut Setting, faces: Faces, report: &Report, tabs: &[(usize, String)]) -> Self {
        let [size, _] = fig::TITLE_TYPE;
        let mut block = |article: &Article, note: bool| {
            let width = match (article.die, note) {
                (Some(_), _) => fig::BESIDE_DIE,
                (None, _) => fig::COLUMN,
            };
            let measure = if note { fig::NOTE } else { width };
            let paragraphs = if note {
                article.lines.clone()
            } else {
                vec![article.lines.join(" ")]
            };
            Block {
                title: setting.lines(&bound(&article.title), faces.headline, size, width),
                die: article.die,
                paragraphs: paragraphs
                    .iter()
                    .filter(|text| !text.is_empty())
                    .map(|text| {
                        setting.lines(&bound(text), faces.regular, fig::TEXT_TYPE[0], measure)
                    })
                    .collect(),
                width,
                measure,
                below_title: fig::BELOW_TITLE[usize::from(note)],
            }
        };
        let mut blocks: Vec<Block> = report.articles.iter().map(|a| block(a, false)).collect();
        blocks.extend(report.notes.iter().map(|note| block(note, true)));
        let mut sheet = Self {
            tabs: rows(setting, faces, tabs),
            headline: setting.balanced(
                &bound(&report.headline),
                faces.headline,
                fig::HEADLINE_TYPE[0],
                fig::FULL,
            ),
            summary: report.summary.as_ref().map_or(vec![], |summary| {
                setting.balanced(
                    &bound(summary),
                    faces.regular,
                    fig::SUMMARY_TYPE[0],
                    fig::SUMMARY[1],
                )
            }),
            blocks,
            pages: vec![],
        };
        let heights: Vec<f32> = sheet.blocks.iter().map(Block::height).collect();
        let tallest = [0, 1].map(|page| {
            fig::TALLEST - fig::FOOT - fig::ABOVE_FOOT - fig::BELOW_COLUMNS - sheet.columns(page)
        });
        sheet.pages = pages(&heights, tallest);
        sheet
    }

    /// How far the tabs move what is below the folio line.
    fn band(&self) -> f32 {
        match self.tabs.len() {
            0 => 0.,
            rows => fig::BAND + (rows - 1) as f32 * fig::TAB_ROW,
        }
    }
    fn summary_top(&self) -> f32 {
        let lines = self.headline.len() as f32;
        fig::HEADLINE + self.band() + lines * fig::HEADLINE_TYPE[1] + fig::ABOVE_SUMMARY
    }
    /// The rule below the headline and the summary.
    fn headline_rule(&self) -> f32 {
        let words = match self.summary.len() {
            0 => self.summary_top() - fig::ABOVE_SUMMARY,
            lines => self.summary_top() + lines as f32 * fig::SUMMARY_TYPE[1],
        };
        words + fig::ABOVE_RULE
    }
    /// Where the columns start on `page`: the headline is on the first.
    fn columns(&self, page: usize) -> f32 {
        let rule = match page {
            0 => self.headline_rule(),
            _ => fig::FOLIO_RULE + self.band(),
        };
        rule + fig::BELOW_RULE
    }
    fn rules_end(&self, page: usize) -> f32 {
        self.columns(page) + self.pages[page].height + fig::BELOW_COLUMNS
    }
    fn foot(&self, page: usize) -> f32 {
        self.rules_end(page) + fig::ABOVE_FOOT
    }
    fn height(&self, page: usize) -> f32 {
        self.foot(page) + fig::FOOT
    }
}

/// The tabs in rows that fit the page. Many tabs say only their numbers.
fn rows(
    setting: &mut Setting,
    faces: Faces,
    tabs: &[(usize, String)],
) -> Vec<Vec<(usize, String, f32)>> {
    const SHORT: usize = 5;
    let mut rows: Vec<Vec<(usize, String, f32)>> = vec![];
    let mut left = 0.;
    for (number, words) in tabs {
        let words = if tabs.len() > SHORT {
            format!("No. {number}")
        } else {
            words.clone()
        };
        let text = setting.width(&words, faces.bold, fig::FOLIO_TYPE[0]);
        let width = text.round() + 2. * fig::TAB_PAD;
        if rows.is_empty() || left + width > fig::FULL {
            rows.push(vec![]);
            left = 0.;
        }
        left += width + fig::TAB_GAP;
        rows.last_mut()
            .expect("A row was started")
            .push((*number, words, width));
    }
    rows
}

/// What the page that is drawn is: the edition's number and how it is open.
#[derive(Resource, Default)]
pub(super) struct Drawn {
    page: Option<(usize, Open)>,
    /// How many pages that edition has.
    pages: usize,
    window: Vec2,
}

/// The open edition's entities.
#[derive(Component)]
pub(super) struct Root;

/// Names a drawn part by its node name in `atlas-newspaper.figcraft`, with its words, their
/// size in logical window pixels and their lines.
#[derive(Component)]
pub(super) struct Part {
    name: String,
    words: Option<(String, f32, usize)>,
    /// The exported image it draws, by its file name.
    image: Option<String>,
}

/// The words of the named part, which its box is wider than.
#[derive(Component)]
pub(super) struct Ink(String);

/// Draws on the page, whose frame unit is `u` window pixels.
struct Printer<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    art: &'a Art,
    page: Entity,
    u: f32,
}

/// Type: its face, size and line height, colour and justification.
#[derive(Clone, Copy)]
struct Type<'a> {
    face: &'a Handle<Font>,
    size: [f32; 2],
    color: Color,
    justify: Justify,
}

impl<'a> Type<'a> {
    fn new(face: &'a Handle<Font>, size: [f32; 2], color: Color, justify: Justify) -> Self {
        Self {
            face,
            size,
            color,
            justify,
        }
    }
}

impl Printer<'_, '_, '_> {
    fn node(&self, [x, y, w, h]: [f32; 4]) -> Node {
        let u = self.u;
        Node {
            position_type: PositionType::Absolute,
            left: px(x * u),
            top: px(y * u),
            width: px(w * u),
            height: px(h * u),
            ..default()
        }
    }
    fn image(&mut self, name: &str, file: &str, rect: [f32; 4], mode: NodeImageMode) -> Entity {
        let node = self.node(rect);
        let image = ImageNode::new(self.art.images[file].clone()).with_mode(mode);
        self.commands
            .spawn((
                image,
                node,
                Part {
                    name: name.into(),
                    words: None,
                    image: Some(file.into()),
                },
                Pickable::IGNORE,
                ChildOf(self.page),
            ))
            .id()
    }
    fn rule(&mut self, name: &str, file: &str, rect: [f32; 4]) {
        self.image(name, file, rect, NodeImageMode::Stretch);
    }
    /// `lines` at `[left, top, width]`, broken as they are: a box as wide as the layout's,
    /// with the words at the side of it that their type says.
    fn words(&mut self, name: &str, lines: &[String], [x, y, w]: [f32; 3], set: Type) {
        let [size, line] = set.size;
        let mut node = self.node([x, y, w, line * lines.len() as f32]);
        node.justify_content = match set.justify {
            Justify::Right => JustifyContent::End,
            Justify::Center => JustifyContent::Center,
            _ => JustifyContent::Start,
        };
        let text = lines.join("\n");
        let part = Part {
            name: name.into(),
            words: Some((plain(&text), size * self.u, lines.len())),
            image: None,
        };
        let boxed = self
            .commands
            .spawn((node, part, Pickable::IGNORE, ChildOf(self.page)))
            .id();
        self.commands.spawn((
            Text::new(text),
            TextFont {
                font: set.face.clone().into(),
                font_size: (size * self.u).into(),
                ..default()
            },
            LineHeight::Px(line * self.u),
            TextLayout {
                justify: set.justify,
                linebreak: LineBreak::NoWrap,
            },
            TextColor(set.color),
            Ink(name.into()),
            Pickable::IGNORE,
            ChildOf(boxed),
        ));
    }
    fn line(&mut self, name: &str, text: &str, at: [f32; 3], set: Type) {
        self.words(name, &[text.to_owned()], at, set);
    }
}

/// Draws the open edition over everything, when what it shows changes.
#[allow(clippy::too_many_arguments)] // A Bevy system: its parameters are the resources it reads.
pub(super) fn draw(
    mut commands: Commands,
    art: Res<Art>,
    ui: Res<Interface>,
    fonts: Res<Assets<Font>>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Root>>,
    mut drawn: ResMut<Drawn>,
) {
    let size = Vec2::new(window.width(), window.height());
    let wanted = ui
        .editions
        .open()
        .map(|(open, edition)| (edition.number, open));
    if drawn.page == wanted && (drawn.window == size || wanted.is_none()) {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    *drawn = Drawn {
        page: wanted,
        pages: 0,
        window: size,
    };
    let Some((open, edition)) = ui.editions.open() else {
        return;
    };
    let tabs: Vec<(usize, String)> = if open.reopened && ui.editions.pile() > 1 {
        ui.editions
            .latest_first()
            .map(|edition| (edition.number, tab(edition)))
            .collect()
    } else {
        vec![]
    };
    let mut setting = Setting {
        fonts: &fonts,
        font_cx: &mut font_cx,
        layout_cx: &mut layout_cx,
    };
    let sheet = Sheet::set(&mut setting, Faces::of(&art), &edition.report, &tabs);
    drawn.pages = sheet.pages.len();
    print(&mut commands, &art, &window, edition, open, &sheet);
}

/// Spawns the scrim, the page and what is printed on it.
fn print(
    commands: &mut Commands,
    art: &Art,
    window: &Window,
    edition: &Edition,
    open: Open,
    sheet: &Sheet,
) {
    let u = reference_pixel(window) * HEIGHT / fig::FRAME[1];
    let report = &edition.report;
    let page = open.page.min(sheet.pages.len() - 1);
    // The whole window puts the page away, and nothing beneath it takes the pointer.
    commands
        .spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.),
                top: px(0.),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GlobalZIndex(LAYER),
            Control {
                name: NAME.into(),
                choice: false,
                action: Action::Edition(Press::PutAway),
            },
        ))
        .observe(click);
    commands.spawn((
        Root,
        ImageNode::new(art.images["atlas-newspaper-scrim.png"].clone())
            .with_mode(NodeImageMode::Stretch),
        Node {
            position_type: PositionType::Absolute,
            left: px(0.),
            top: px(fig::SCRIM_TOP * u),
            width: percent(100),
            bottom: px(0.),
            ..default()
        },
        GlobalZIndex(LAYER),
        Part {
            name: "edition-scrim".into(),
            words: None,
            image: None,
        },
        Pickable::IGNORE,
    ));
    let [x, y, w] = fig::PAGE;
    let height = sheet.height(page);
    let root = commands
        .spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                left: px(window.width() / 2. + (x - fig::FRAME[0] / 2.) * u),
                top: px(y * u),
                width: px(w * u),
                height: px(height * u),
                ..default()
            },
            // Over the pile's three layers.
            GlobalZIndex(LAYER + 4),
            Part {
                name: "edition-page".into(),
                words: None,
                image: None,
            },
            Pickable::IGNORE,
        ))
        .id();
    let [l, t, r, b] = fig::SHADOW.map(|reach| px(-reach * u));
    commands.spawn((
        ImageNode::new(art.images[PAGE_PLATE].clone()).with_mode(sliced(PAGE_SLICE, u)),
        Node {
            position_type: PositionType::Absolute,
            left: l,
            top: t,
            right: r,
            bottom: b,
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(root),
    ));
    let mut on = Printer {
        commands,
        art,
        page: root,
        u,
    };
    let (regular, bold, head) = (&art.font, &art.bold, &art.headline);
    let text = Type::new;
    on.rule(
        "edition-masthead",
        "atlas-newspaper-masthead.png",
        fig::MASTHEAD,
    );
    let folio_type = text(bold, fig::FOLIO_TYPE, INK, Justify::Left);
    on.line("edition-folio", &folio(edition), fig::FOLIO, folio_type);
    let about_type = text(regular, fig::FOLIO_TYPE, INK, Justify::Right);
    on.line("edition-about", &about(report), fig::ABOUT, about_type);
    tabs(&mut on, sheet, edition.number);
    let band = sheet.band();
    let across = |y: f32| [fig::PAD, y, fig::FULL, fig::RULE];
    on.rule(
        "edition-folio-rule",
        RULE_INK,
        across(fig::FOLIO_RULE + band),
    );
    let top = sheet.columns(page);
    if page == 0 {
        on.words(
            "edition-headline",
            &sheet.headline,
            [fig::PAD, fig::HEADLINE + band, fig::FULL],
            text(head, fig::HEADLINE_TYPE, INK, Justify::Center),
        );
        if !sheet.summary.is_empty() {
            let [left, width] = fig::SUMMARY;
            on.words(
                "edition-summary",
                &sheet.summary,
                [left, sheet.summary_top(), width],
                text(regular, fig::SUMMARY_TYPE, INK, Justify::Center),
            );
        }
        on.rule(
            "edition-headline-rule",
            RULE_INK,
            across(sheet.headline_rule()),
        );
        standings(&mut on, &report.standings, top);
    }
    for (i, left) in fig::COLUMN_RULES.into_iter().enumerate() {
        let tall = sheet.rules_end(page) - top;
        on.rule(
            &format!("edition-column-rule-{i}"),
            RULE,
            [left, top, fig::RULE, tall],
        );
    }
    for stands in &sheet.pages[page].stands {
        block(&mut on, sheet, *stands, top);
    }
    let foot_y = sheet.foot(page);
    let foot_type = |justify| text(regular, fig::FOOT_TYPE, QUIET, justify);
    let at = |[left, width]: [f32; 2]| [left, foot_y, width];
    on.line(
        "edition-source",
        SOURCE,
        at(fig::SOURCE),
        foot_type(Justify::Left),
    );
    on.line(
        "edition-away",
        AWAY,
        at(fig::AWAY),
        foot_type(Justify::Right),
    );
    if let Some(words) = foot(page, sheet.pages.len()) {
        on.line(
            "edition-pages",
            &words,
            at(fig::PAGES),
            foot_type(Justify::Center),
        );
    }
}

/// A reopened edition's tabs: a click on another's turns to it and decides nothing.
fn tabs(on: &mut Printer, sheet: &Sheet, open: usize) {
    for (row, tabs) in sheet.tabs.iter().enumerate() {
        let top = fig::TABS + row as f32 * fig::TAB_ROW;
        let mut left = fig::PAD;
        for (number, words, width) in tabs {
            let is_open = *number == open;
            let name = format!("edition-tab:{number}");
            let plate = on.image(
                &name,
                TAB_PLATES[usize::from(is_open)],
                [left, top, *width, fig::TAB],
                sliced(TAB_SLICE, on.u),
            );
            on.commands
                .entity(plate)
                .remove::<Pickable>()
                .insert(Control {
                    name: name.clone(),
                    choice: false,
                    action: Action::Edition(Press::Show(*number)),
                })
                .observe(click);
            let face = on.art.bold.clone();
            let ink = if is_open { PAPER } else { INK };
            let inset = (fig::TAB - fig::FOLIO_TYPE[1]) / 2. - 1.;
            on.line(
                &format!("{name}-label"),
                words,
                [left, top + inset, *width],
                Type::new(&face, fig::FOLIO_TYPE, ink, Justify::Center),
            );
            left += width + fig::TAB_GAP;
        }
    }
}

/// The table of Resources and margins at the top of the third column.
fn standings(on: &mut Printer, rows: &[Row], top: f32) {
    let left = fig::COLUMNS[2];
    let (regular, bold, head) = (&on.art.font, &on.art.bold, &on.art.headline);
    let text = Type::new;
    let at = |[x, w]: [f32; 2], y: f32| [left + x, y, w];
    on.line(
        "edition-standings",
        STANDINGS,
        [left, top, fig::TABLE],
        text(head, fig::TITLE_TYPE, INK, Justify::Left),
    );
    let heading = text(bold, fig::HEADING_TYPE, QUIET, Justify::Right);
    let y = top + fig::HEADINGS;
    for (what, words, place) in [
        ("resources", "RESOURCES", fig::RESOURCES),
        ("margin", "MARGIN", fig::MARGIN),
    ] {
        on.line(
            &format!("edition-heading:{what}"),
            words,
            at(place, y),
            heading,
        );
    }
    for (i, row) in rows.iter().enumerate() {
        let y = top + fig::ROWS + i as f32 * fig::ROW;
        on.rule(
            &format!("edition-row-rule-{i}"),
            RULE,
            [left, y, fig::TABLE, fig::RULE],
        );
        let faction = slug(row.faction);
        let [dx, dy, size] = fig::EMBLEM;
        on.image(
            &format!("edition-emblem:{faction}"),
            &emblems::image(row.faction),
            [left + dx, y + dy, size, size],
            NodeImageMode::Stretch,
        );
        let words = y + fig::ROW_TEXT;
        on.line(
            &format!("edition-faction:{faction}"),
            row.faction.name(),
            at(fig::NAME, words),
            text(regular, fig::ROW_TYPE, INK, Justify::Left),
        );
        for (what, of, place) in [
            ("resources", row.resources, fig::RESOURCES),
            ("margin", row.margin, fig::MARGIN),
        ] {
            let (said, changed) = figure(of);
            let (face, color) = if changed {
                (bold, INK)
            } else {
                (regular, QUIET)
            };
            on.line(
                &format!("edition-{what}:{faction}"),
                &said,
                at(place, words),
                text(face, fig::ROW_TYPE, color, Justify::Right),
            );
        }
    }
    on.rule(
        &format!("edition-row-rule-{}", rows.len()),
        RULE,
        [
            left,
            top + fig::ROWS + rows.len() as f32 * fig::ROW,
            fig::TABLE,
            fig::RULE,
        ],
    );
}

fn block(on: &mut Printer, sheet: &Sheet, stands: Stands, top: f32) {
    let block = &sheet.blocks[stands.block];
    let (left, top) = (fig::COLUMNS[stands.column], top + stands.top);
    let name = format!("edition-block-{}", stands.block);
    let (head, regular) = (on.art.headline.clone(), on.art.font.clone());
    on.words(
        &format!("{name}-title"),
        &block.title,
        [left, top, block.width],
        Type::new(&head, fig::TITLE_TYPE, INK, Justify::Left),
    );
    if let Some(die) = block.die {
        let [dx, size] = fig::DIE;
        on.image(
            &format!("{name}-die"),
            &format!("atlas-newspaper-die-{die}.png"),
            [left + dx, top, size, size],
            NodeImageMode::Stretch,
        );
    }
    let mut y = top + block.text_top();
    for (i, lines) in block.paragraphs.iter().enumerate() {
        on.words(
            &format!("{name}-text-{i}"),
            lines,
            [left, y, block.measure],
            Type::new(&regular, fig::TEXT_TYPE, INK, Justify::Left),
        );
        y += lines.len() as f32 * fig::TEXT_TYPE[1] + fig::BETWEEN_SENTENCES;
    }
}

/// Draws the pile of the card's editions in the main view's `editions` area, over the
/// transfer modal's scrim and the open edition's.
pub(super) fn pile(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    ui: &Interface,
    layout: &areas::Layout,
    window: &Window,
) {
    let count = ui.editions.pile();
    if count == 0 {
        return;
    }
    let modal = transfer::shown(view.transfer.as_ref(), view.owner, ui);
    let z = if ui.editions.open().is_some() {
        LAYER + 1
    } else if modal {
        transfer::LAYER
    } else {
        2
    };
    let area = layout.area(AREA, window);
    let u = reference_pixel(window) * HEIGHT / fig::FRAME[1];
    let part = |name: &str, words: Option<(String, f32, usize)>| Part {
        name: name.into(),
        words,
        image: None,
    };
    let pile = picture(
        commands,
        art,
        "atlas-newspaper-pile.png",
        fig::PILE,
        u,
        area.min,
        z,
    );
    commands
        .entity(pile)
        .remove::<Pickable>()
        .insert((
            Control {
                name: PILE.into(),
                choice: false,
                action: Action::Edition(Press::Reopen),
            },
            part("editions", None),
        ))
        .observe(click);
    let disc = picture(
        commands,
        art,
        "atlas-main-view-count-disc.png",
        fig::DISC,
        u,
        area.min,
        z + 1,
    );
    commands.entity(disc).insert(part("editions-disc", None));
    let mut words = |name: &str, text: String, face, color, place: [f32; 4], line: f32| {
        let size = place[3];
        let entity = styled(
            commands,
            art,
            Prose::named(text.as_str()),
            face,
            color,
            place,
            u,
            area.min,
            z + 2,
        );
        commands.entity(entity).insert((
            LineHeight::Px(line * u),
            part(name, Some((text, size * u, 1))),
        ));
        entity
    };
    let [x, y, w] = fig::COUNT;
    let [size, line] = fig::COUNT_TYPE;
    let number = words(
        "editions-count",
        count.to_string(),
        &art.bold,
        PAPER,
        [x, y, w, size],
        line,
    );
    let [x, y] = fig::CAPTION;
    let [size, line] = fig::CAPTION_TYPE;
    words(
        "editions-caption",
        format!("card {}'s editions", view.current_card),
        &art.font,
        QUIET,
        [x, y, fig::PILE[2], size],
        line,
    );
    commands
        .entity(number)
        .insert(TextLayout::justify(Justify::Center));
}

/// The newspaper for the development seat (`table.newspaper`): whether an edition is due,
/// how many lie on the pile, the open one with the report it prints, and every drawn part
/// with its words, their size in logical window pixels, its rectangle there and, for a text
/// that its box is wider than, the words' own (`ink`).
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let ink: BTreeMap<String, [f32; 4]> = world
        .query::<(&Ink, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
        .iter(world)
        .map(|(ink, node, at)| (ink.0.clone(), seat::logical_bounds(node, at)))
        .collect();
    let mut parts = world
        .query::<(&Part, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
        .iter(world)
        .map(|(part, node, at)| {
            let words = part.words.as_ref();
            (
                part.name.clone(),
                json!({
                    "part": part.name,
                    "text": words.map(|(text, ..)| text),
                    "lines": words.map(|(.., lines)| lines),
                    "font_size": words.map(|(_, size, _)| size),
                    "bounds": seat::logical_bounds(node, at),
                    "ink": ink.get(&part.name),
                    "image": part.image,
                }),
            )
        })
        .collect::<Vec<_>>();
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    let editions = &world.resource::<Interface>().editions;
    let pages = world.resource::<Drawn>().pages;
    let open = editions.open().map(|(open, edition)| {
        json!({
            "number": edition.number,
            "reopened": open.reopened,
            "page": open.page + 1,
            "pages": pages,
            "report": edition.report,
        })
    });
    json!({
        "due": editions.due(),
        "pile": editions.pile(),
        "open": open,
        "parts": parts.into_iter().map(|(_, part)| part).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three admitted faces, as the app registers them.
    struct Set {
        app: App,
        faces: [Handle<Font>; 3],
    }

    impl Set {
        fn new() -> Self {
            let mut app = App::new();
            app.add_plugins((
                MinimalPlugins,
                AssetPlugin::default(),
                bevy::image::ImagePlugin::default(),
                bevy::text::TextPlugin,
            ));
            let faces = ["Lato-Regular", "Lato-Bold", "NotoSerifDisplay-Bold"].map(|file| {
                let bytes = std::fs::read(format!("art/atlas/{file}.ttf")).unwrap();
                app.world_mut()
                    .resource_mut::<Assets<Font>>()
                    .add(Font::from_bytes(bytes))
            });
            app.update();
            Self { app, faces }
        }
        fn with<T>(&mut self, set: impl FnOnce(&mut Setting, Faces) -> T) -> T {
            let [regular, bold, headline] = &self.faces;
            let faces = Faces {
                regular,
                bold,
                headline,
            };
            self.app
                .world_mut()
                .resource_scope(|world, fonts: Mut<Assets<Font>>| {
                    world.resource_scope(|world, mut font_cx: Mut<FontCx>| {
                        let mut layout_cx = world.resource_mut::<LayoutCx>();
                        let mut setting = Setting {
                            fonts: &fonts,
                            font_cx: &mut font_cx,
                            layout_cx: &mut layout_cx,
                        };
                        set(&mut setting, faces)
                    })
                })
        }
    }

    fn record() -> serde_json::Value {
        serde_json::from_str(include_str!("../../art/atlas/atlas-newspaper.layout.json")).unwrap()
    }

    /// The last report of a match played from `seed` by `decisions`.
    fn report(seed: u64, decisions: &[String]) -> Report {
        let mut game = Game::opening_with_seed(seed);
        let mut reports = crate::reports::Reports::new(&game.observe());
        for decision in decisions {
            game.apply(decision).unwrap();
            reports.record(&game.observe());
        }
        reports.all().pop().unwrap()
    }

    /// The layout's samples: their frame, and the report they print.
    fn samples() -> [(&'static str, Report); 2] {
        let decisions = |frame: &str| -> Vec<String> {
            serde_json::from_value(record()[frame]["decisions"].clone()).unwrap()
        };
        let terror = decisions("terror");
        let mut pass = terror.clone();
        pass.extend(decisions("pass"));
        [("terror", report(13, &terror)), ("pass", report(13, &pass))]
    }

    /// Set from the admitted fonts, each sample has the layout's lines, and its blocks,
    /// rules and foot stand where the layout's frame has them.
    #[test]
    fn the_samples_are_set_as_the_layout_sets_them() {
        let mut set = Set::new();
        for (frame, report) in samples() {
            let sheet = set.with(|setting, faces| Sheet::set(setting, faces, &report, &[]));
            let record = record();
            let rect = |group: &str, name: &str| -> [f32; 4] {
                serde_json::from_value(record[frame][group][name].clone())
                    .unwrap_or_else(|e| panic!("{frame} {name}: {e}"))
            };
            let [px, py, pw, ph] = rect("parts", "edition-page");
            assert_eq!([px, py, pw], fig::PAGE);
            assert_eq!(py + fig::TALLEST, fig::FRAME[1] - 42.);
            assert_eq!(sheet.pages.len(), 1);
            assert_eq!(ph, sheet.height(0), "{frame}");
            assert_eq!(rect("parts", "edition-folio-rule")[1] - py, fig::FOLIO_RULE);
            assert_eq!(
                rect("parts", "edition-headline-rule")[1] - py,
                sheet.headline_rule()
            );
            let tall = |lines: usize, [_, line]: [f32; 2]| lines as f32 * line;
            let headline = rect("text", "edition-headline");
            assert_eq!(headline[3], tall(sheet.headline.len(), fig::HEADLINE_TYPE));
            // Balanced, as the frame's are.
            if frame == "terror" {
                assert_eq!(
                    sheet.headline,
                    ["26July executes Terror in", "Oriente and Santiago de Cuba"]
                );
                assert_eq!(
                    sheet.summary.iter().map(|l| plain(l)).collect::<Vec<_>>(),
                    [
                        "Kidnap in Santiago de Cuba: the die shows 2,",
                        "and 26July takes 2 Resources from Government."
                    ]
                );
            }
            let summary = rect("text", "edition-summary");
            assert_eq!(summary[3], tall(sheet.summary.len(), fig::SUMMARY_TYPE));
            assert_eq!(summary[1] - py, sheet.summary_top());
            let top = sheet.columns(0);
            for (i, left) in fig::COLUMN_RULES.into_iter().enumerate() {
                let expected = [px + left, py + top, fig::RULE, sheet.rules_end(0) - top];
                assert_eq!(rect("parts", &format!("edition-column-rule-{i}")), expected);
            }
            assert_eq!(rect("text", "edition-source")[1] - py, sheet.foot(0));
            assert_eq!(rect("text", "edition-standings")[1] - py, top);
            for stands in &sheet.pages[0].stands {
                let block = &sheet.blocks[stands.block];
                let name = format!("edition-block-{}", stands.block);
                let (x, y) = (px + fig::COLUMNS[stands.column], py + top + stands.top);
                let title = tall(block.title.len(), fig::TITLE_TYPE);
                assert_eq!(
                    rect("text", &format!("{name}-title")),
                    [x, y, block.width, title],
                    "{frame} {name}"
                );
                let mut y = y + block.text_top();
                for (i, lines) in block.paragraphs.iter().enumerate() {
                    let text = tall(lines.len(), fig::TEXT_TYPE);
                    assert_eq!(
                        rect("text", &format!("{name}-text-{i}")),
                        [x, y, block.measure, text],
                        "{frame} {name}"
                    );
                    y += text + fig::BETWEEN_SENTENCES;
                }
            }
            assert_eq!(
                sheet.pages[0].stands.len(),
                report.articles.len() + report.notes.len()
            );
        }
    }

    /// Every export the page places is the record's, with its size, and admitted, as its
    /// face is.
    #[test]
    fn the_exports_are_the_record_s_and_admitted() {
        let manifest = crate::assets::NEWSPAPER_MANIFEST;
        let exports = record()["exports"].as_object().unwrap().clone();
        for (file, size) in &exports {
            let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
            let image = image::load_from_memory(&bytes).unwrap();
            let size: [u32; 2] = serde_json::from_value(size.clone()).unwrap();
            assert_eq!([image.width(), image.height()], size, "{file}");
            assert!(manifest.contains(&format!("art/atlas/{file}")), "{file}");
        }
        assert!(manifest.contains("art/atlas/NotoSerifDisplay-Bold.ttf"));
        assert_eq!(manifest.lines().count(), exports.len() + 1);
        // The plates' sizes hold their slices, at 2 px per unit.
        let [_, t, _, b] = fig::SHADOW;
        let shadow = fig::SHADOW[0].max(t).max(b);
        assert!(PAGE_SLICE > shadow && 2. * PAGE_SLICE < 484.);
    }

    /// While an edition is due, before it opens too, the human submits nothing, and is
    /// told so.
    #[test]
    fn nothing_is_submitted_while_an_edition_is_due() {
        let dir = tempfile::tempdir().unwrap();
        let mut play = test_play(dir.path().join("match.json"));
        let mut ui = Interface::default();
        let seen = play.game().observe();
        ui.editions = crate::editions::Editions::from(&seen);
        ui.seats[Faction::M26.index()] = crate::faction_seats::SeatState::Thinking;
        submit(&mut play, &mut ui, Faction::M26, 0, "0:pass").unwrap();
        assert!(ui.editions.due() && ui.editions.open().is_none());
        ui.seats = default();
        let mut undecided = test_play(dir.path().join("other.json"));
        decide(&mut undecided, &mut ui, Faction::M26, 0, "0:terror");
        assert_eq!(undecided.game().observe().revision, 0);
        let refusal = notice::Notice::refusal(notice::EDITION_FIRST);
        assert_eq!(ui.notice, refusal);
        ui.notice = default();
        ui.editions.open_next();
        decide(&mut undecided, &mut ui, Faction::M26, 0, "0:terror");
        assert_eq!(undecided.game().observe().revision, 0);
        assert_eq!(ui.notice, refusal);
        ui.editions.put_away();
        decide(&mut undecided, &mut ui, Faction::M26, 0, "0:terror");
        assert_eq!(undecided.game().observe().revision, 1);
    }

    /// The longest editions found: each prints every block once, in order, within the
    /// page, and only the unshaded Event's needs a second one.
    #[test]
    fn the_longest_editions_are_set_on_one_page_or_two() {
        let mut set = Set::new();
        let found: Vec<_> = crate::longest::longest()
            .iter()
            .map(|decisions| {
                let report = report(crate::SEED, decisions);
                let sheet = set.with(|setting, faces| Sheet::set(setting, faces, &report, &[]));
                let blocks: Vec<usize> = sheet
                    .pages
                    .iter()
                    .flat_map(|page| page.stands.iter().map(|stands| stands.block))
                    .collect();
                assert!(blocks.iter().copied().eq(0..sheet.blocks.len()));
                let heights: Vec<f32> = (0..sheet.pages.len()).map(|p| sheet.height(p)).collect();
                assert!(heights.iter().all(|height| *height <= fig::TALLEST));
                heights
            })
            .collect();
        assert_eq!(
            found,
            [vec![1898.], vec![1576.], vec![1886., 604.], vec![1862.]]
        );
    }

    /// No edition can be cut: whatever the blocks' heights, each stands once, in order,
    /// clear of the standings and of each other, and no page is taller than the tallest
    /// but for a block taller than a column.
    #[test]
    fn every_block_stands_once_and_in_its_column() {
        let tallest = [1224., 1506.];
        let mut seed = 7_u32;
        let mut next = |below: u32| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) % below
        };
        let mut several = 0;
        for _ in 0..400 {
            let count = next(40);
            let mut heights: Vec<f32> = (0..count).map(|_| 80. + next(400) as f32).collect();
            if next(10) == 0 {
                heights.push(2000.);
            }
            let pages = pages(&heights, tallest);
            several += usize::from(pages.len() > 1);
            let mut stood = vec![];
            for (i, page) in pages.iter().enumerate() {
                let tallest = tallest[usize::from(i > 0)];
                let mut foot = [0., 0., if i == 0 { 328. } else { 0. }];
                let mut column = 0;
                for stands in &page.stands {
                    assert!(stands.column >= column, "{page:?}");
                    column = stands.column;
                    assert!(stands.top >= foot[column], "{page:?}");
                    let end = stands.top + heights[stands.block];
                    assert!(end <= page.height.max(tallest.min(end)), "{page:?}");
                    assert!(end <= tallest || stands.top == 0., "{page:?}");
                    foot[column] = end + fig::BETWEEN;
                    stood.push(stands.block);
                }
                assert!(i == 0 || !page.stands.is_empty());
            }
            assert!(stood.iter().copied().eq(0..heights.len()), "{heights:?}");
        }
        assert!(several > 100, "{several}");
        // Nothing to tell but the standings.
        assert_eq!(
            pages(&[], tallest),
            [Page {
                stands: vec![],
                height: fig::STANDINGS
            }]
        );
    }

    #[test]
    fn a_number_stays_with_the_word_it_counts() {
        let lines = Set::new().with(|setting, faces| {
            setting.lines(
                &bound("Syndicate gives 26July 2 Resources"),
                faces.headline,
                fig::TITLE_TYPE[0],
                fig::COLUMN,
            )
        });
        assert_eq!(lines, ["Syndicate gives 26July", "2\u{a0}Resources"]);
        let bound = bound("26July 4 → 3. 1 Terror marker in 12 spaces: Card 18.");
        assert_eq!(
            bound.replace('\u{a0}', "_"),
            "26July 4_→_3. 1_Terror marker in 12_spaces: Card 18."
        );
        assert_eq!(
            plain(&bound),
            "26July 4 → 3. 1 Terror marker in 12 spaces: Card 18."
        );
    }

    #[test]
    fn the_folio_line_and_the_tabs_say_which_edition_it_is() {
        let [(_, terror), (_, pass)] = samples();
        let edition = |report: &Report, number| Edition {
            number,
            report: report.clone(),
        };
        assert_eq!(
            folio(&edition(&terror, 1)),
            "No. 1 · Card 17 · Alberto Bayo"
        );
        assert_eq!(
            about(&terror),
            "26July · 1st Eligible · Operation with Special Activity"
        );
        assert_eq!(tab(&edition(&terror, 1)), "No. 1 · Terror ends");
        assert_eq!(about(&pass), "Government · 1st Eligible · Pass");
        assert_eq!(tab(&edition(&pass, 3)), "No. 3 · Pass");
        let partway = report(13, &["terror", "transfer", "ask:0:2"].map(String::from));
        assert_eq!(
            folio(&edition(&partway, 2)),
            "No. 2 · partway · Card 17 · Alberto Bayo"
        );
        assert_eq!(
            about(&partway),
            "26July · 1st Eligible · Terror in progress"
        );
        assert_eq!(tab(&edition(&partway, 2)), "No. 2 · partway");
        // No box is recorded for an Event.
        let mut event = crate::longest::HANDOFF.map(String::from).to_vec();
        event.extend(["event:shaded", "resolve-shaded"].map(String::from));
        assert_eq!(
            about(&report(13, &event)),
            "Directorio · 2nd Eligible · Event"
        );
        assert_eq!(foot(0, 1), None);
        assert_eq!(foot(1, 2).unwrap(), "Page 2 of 2 · Left and Right turn it");
    }

    /// A card's many editions have rows of tabs that say their numbers, each within the
    /// page, and the rest moves down by the rows.
    #[test]
    fn many_tabs_stand_in_rows() {
        let mut set = Set::new();
        let [(_, terror), _] = samples();
        for (count, rows, band) in [(2, 1, 74.), (5, 1, 74.), (6, 1, 74.), (30, 3, 210.)] {
            let tabs: Vec<(usize, String)> = (1..=count)
                .rev()
                .map(|number| (number, format!("No. {number} · Terror ends")))
                .collect();
            let sheet = set.with(|setting, faces| Sheet::set(setting, faces, &terror, &tabs));
            assert_eq!(sheet.tabs.len(), rows, "{count}");
            assert_eq!(sheet.band(), band);
            let mut seen = vec![];
            for row in &sheet.tabs {
                let wide: f32 = row.iter().map(|(.., width)| width + fig::TAB_GAP).sum();
                assert!(wide - fig::TAB_GAP <= fig::FULL, "{wide}");
                seen.extend(row.iter().map(|(number, ..)| *number));
                assert_eq!(row[0].1.contains("Terror"), count <= 5);
            }
            assert!(seen.into_iter().eq((1..=count).rev()));
            assert!((0..sheet.pages.len()).all(|page| sheet.height(page) <= fig::TALLEST));
        }
    }
}
