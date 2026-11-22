//! Original starter presentations for File › New.
//!
//! Each deck is a few slides of our own text on one of our themes. Nothing here is copied from
//! another product's templates.

use std::sync::Arc;

use deckcraft_geom::{Size, Xfrm};
use deckcraft_model::style::PictureFill;
use deckcraft_model::text::{Action, Hyperlink, TextBody};
use deckcraft_model::{PhType, Presentation, Shape, ShapeId, ShapeKind, defaults};
use serde_json::Value;

use crate::Result;
use crate::cmd::{bad, slide::find_layout};

/// One slide of a starter: a layout, a title, text for each non-title placeholder in order, and
/// optional speaker notes.
pub struct SlideSeed {
    pub layout: &'static str,
    pub title: &'static str,
    pub bodies: &'static [&'static str],
    pub notes: &'static str,
}

/// A starter deck shown in the template gallery.
pub struct Starter {
    pub name: &'static str,
    pub summary: &'static str,
    pub theme: &'static str,
    pub slides: &'static [SlideSeed],
}

const fn seed(layout: &'static str, title: &'static str, bodies: &'static [&'static str], notes: &'static str) -> SlideSeed {
    SlideSeed { layout, title, bodies, notes }
}

const STARTERS: &[Starter] = &[
    Starter {
        name: "Briefing",
        summary: "A weekly check-in: what changed, what's stuck, and who does the next thing",
        theme: "Harbor",
        slides: &[
            seed("title", "Weekly briefing", &["What changed, what's stuck, and what we do next"], "Open with what changed since last week."),
            seed(
                "titleAndContent",
                "This week's agenda",
                &["Shipped since last time\n\tOne concrete change\nStill open\n\tThe decision we need\nNext\n\tWho does it, and by when"],
                "",
            ),
            seed("sectionHeader", "Where we are", &["A short picture before the details"], ""),
            seed(
                "twoContent",
                "Done and next",
                &[
                    "Done\n\tThe change is in people's hands\n\tWe wrote down what we learned",
                    "Next\n\tPick the one risk to retire\n\tSay who owns it",
                ],
                "",
            ),
            seed("title", "See you next week", &["Bring one number and one question"], ""),
        ],
    },
    Starter {
        name: "Lesson",
        summary: "A short class that ends with the learner explaining the idea",
        theme: "Meadow",
        slides: &[
            seed("title", "How a lock works", &["A short lesson for people new to the idea"], ""),
            seed(
                "titleAndContent",
                "By the end",
                &["You can say what a lock protects\n\tAnd what it does not\nYou can sketch one in three steps"],
                "",
            ),
            seed("sectionHeader", "The idea", &["One metaphor, then the real thing"], ""),
            seed(
                "titleAndContent",
                "Three steps",
                &["Name the thing being shared\nDecide who may change it\nLeave it so the next person can tell"],
                "",
            ),
            seed("title", "Try it", &["Explain a lock to someone who has never seen one"], ""),
        ],
    },
    Starter {
        name: "Pitch",
        summary: "A ten-minute conversation about one promise, not a tour of features",
        theme: "Ember",
        slides: &[
            seed("title", "A quieter inbox", &["Ten minutes. One promise."], ""),
            seed(
                "titleAndContent",
                "The mess",
                &[
                    "People miss the one message that mattered\n\tIt arrived between newsletters\nThey spend the morning sorting\n\tAnd still feel behind",
                ],
                "",
            ),
            seed("sectionHeader", "A smaller promise", &["Show the one message. Hide the rest until asked."], ""),
            seed(
                "comparison",
                "Today and after",
                &[
                    "Today",
                    "The important note is buried\nFinding it takes a search",
                    "After",
                    "It sits at the top until you deal with it\nEverything else waits",
                ],
                "",
            ),
            seed("title", "The ask", &["Twenty people, two weeks, one question: did the right note surface?"], ""),
        ],
    },
    Starter {
        name: "Field Notes",
        summary: "What an outing set out to see, and what it actually found",
        theme: "Coral Reef",
        slides: &[
            seed("title", "Saturday on the ridge", &["What we went to see, and what we found"], ""),
            seed("titleAndContent", "Plan", &["Leave at dawn\n\tCold, clear, little wind\nWalk the west path\nBe back before dusk"], ""),
            seed("sectionHeader", "What we saw", &["Notes taken on the spot"], ""),
            seed(
                "twoContent",
                "Expected and found",
                &[
                    "Expected\n\tA clear view from the ridge\n\tThe creek on the map",
                    "Found\n\tLow cloud on the ridge\n\tThe creek, and a better pencil",
                ],
                "",
            ),
            seed("title", "Pack list for next time", &["Warmer gloves. A second pencil. The map that shows the creek."], ""),
        ],
    },
    Starter {
        name: "Quarterly",
        summary: "A report meant to be read aloud in half an hour",
        theme: "Paper",
        slides: &[
            seed("title", "Spring report", &["Read aloud in thirty minutes"], ""),
            seed(
                "titleAndContent",
                "Headlines",
                &["The waiting list got shorter\n\tBy the amount we promised\nTwo things slipped\n\tBoth have a new date"],
                "",
            ),
            seed("sectionHeader", "The numbers", &["Only the ones that change a decision"], ""),
            seed("titleAndContent", "What we will stop", &["The report nobody reads\nThe meeting that repeats the report"], ""),
            seed("title", "Questions we want", &["Where should the next quarter refuse work?"], ""),
        ],
    },
    Starter {
        name: "Evening Talk",
        summary: "A public talk the audience can check on the way home",
        theme: "Nocturne",
        slides: &[
            seed("title", "Lights out", &["A talk about how cities sleep"], ""),
            seed("titleAndContent", "Road map", &["Why the sky looks orange\nWhat a dark street is for\nOne change a block can make"], ""),
            seed("sectionHeader", "Look up", &["The part you can check on the way home"], ""),
            seed(
                "titleAndContent",
                "One change",
                &["Shield the lamp so light goes down\n\tNot into bedroom windows\nLeave a path you can still walk"],
                "",
            ),
            seed("title", "Walk home", &["Count the lamps you can see from your pillow"], ""),
        ],
    },
    Starter {
        name: "Photo Album",
        summary: "A curated collection of photos with captions and clean border presentation",
        theme: "Harbor",
        slides: &[
            seed("title", "Photo Album", &["Curated visual memories and field captures"], ""),
            seed("pictureWithCaption", "Mountain Ridge View", &["Captured at first light above the western valley"], ""),
            seed("pictureWithCaption", "Coastal Inlet Study", &["Tidal currents observed during the seasonal survey"], ""),
            seed("title", "Album Complete", &["All high-resolution plates archived"], ""),
        ],
    },
    Starter {
        name: "Interactive Quiz",
        summary: "An interactive assessment with question branching, feedback, and score navigation",
        theme: "Ember",
        slides: &[
            seed("title", "Interactive Knowledge Check", &["Test your understanding of native document architecture"], "Click Start to begin."),
            seed("titleAndContent", "Question 1: Core Design", &["What is the fundamental storage model of TZWorkQuint?"], ""),
            seed("titleAndContent", "Correct: Offline Sovereignty", &["Exactly right. All files are stored directly on your local system with standard formats.\n\nProceed to the next question."], ""),
            seed("titleAndContent", "Incorrect: Try Again", &["That choice is not correct. TZWorkQuint is built on native offline file sovereignty.\n\nReturn to Question 1 to retry."], ""),
            seed("titleAndContent", "Question 2: Document Portability", &["Which format does Wordloom prioritize for rich interchange?"], ""),
            seed("titleAndContent", "Correct: Open DOCX Fidelity", &["Correct! Wordloom uses standard OOXML DOCX with high fidelity roundtrip.\n\nReview the final summary."], ""),
            seed("titleAndContent", "Incorrect: Try Again", &["Not quite. Wordloom prioritizes standard OOXML DOCX interchange.\n\nReturn to Question 2 to retry."], ""),
            seed("title", "Assessment Complete", &["You have completed the interactive evaluation."], "Thank you for participating."),
        ],
    },
    Starter {
        name: "Training Presentation",
        summary: "A structured course module with objectives, core concepts, hands-on exercise, and key takeaways",
        theme: "Slate",
        slides: &[
            seed("title", "Security & Operations Training", &["Core operational workflows and compliance guidelines"], "Welcome attendees."),
            seed("titleAndContent", "Module Objectives", &["Standardize document versioning across teams\n\tAvoid untracked draft proliferation\nMaster secure local storage practices\n\tEnsure compliance with audit retention rules\nValidate data integrity before sign-off\n\tRun automated parity checks"], ""),
            seed("sectionHeader", "Section 1: Operating Principles", &["Fundamental daily routines"], ""),
            seed("twoContent", "Recommended vs Disallowed", &["Recommended Practices\n\tSave versioned milestones\n\tReview changes before export", "Disallowed Practices\n\tSharing unencrypted credentials\n\tBypassing validation checks"], ""),
            seed("titleAndContent", "Hands-on Exercise", &["Scenario walk-through with team members\n\tEvaluate 3 sample communications\n\tIdentify security warning signs\n\tRecord findings in the log"], ""),
            seed("title", "Training Complete", &["Thank you for participating."], ""),
        ],
    },
];

fn make_link(action: Action, tooltip: &str) -> Hyperlink {
    Hyperlink {
        action,
        tooltip: tooltip.to_string(),
        highlight_click: true,
        sound: None,
        stop_sound: false,
        zoom_return: None,
    }
}

/// The starter gallery, in the order File › New shows it.
pub fn starters() -> &'static [Starter] {
    STARTERS
}

/// Starters whose name, summary, theme or a slide title contains `query` (empty matches all).
pub fn matching(query: &str) -> Vec<&'static Starter> {
    let q = query.trim().to_lowercase();
    let hit = |text: &str| text.to_lowercase().contains(&q);
    starters()
        .iter()
        .filter(|t| q.is_empty() || hit(t.name) || hit(t.summary) || hit(t.theme) || t.slides.iter().any(|slide| hit(slide.title)))
        .collect()
}

pub fn find(name: &str) -> Option<&'static Starter> {
    let name = name.trim();
    starters().iter().find(|t| t.name.eq_ignore_ascii_case(name))
}

fn text_with_levels(text: &str) -> TextBody {
    let mut body = TextBody::from_text(text);
    for para in &mut body.paragraphs {
        let tabs = para.runs.first().map(|r| r.text.chars().take_while(|c| *c == '\t').count()).unwrap_or(0);
        if tabs > 0
            && let Some(run) = para.runs.first_mut()
        {
            run.text = run.text.trim_start_matches('\t').to_string();
            para.level = tabs.min(8) as u8;
        }
    }
    body
}

/// A new presentation filled with `name`'s slides. `size` is the slide size in points.
pub fn build(name: &str, size: Size) -> Result<Presentation> {
    let starter = find(name).ok_or_else(|| {
        let names = starters().iter().map(|t| t.name).collect::<Vec<_>>().join(", ");
        bad("file.new", format!("no template `{name}`; use {names}"))
    })?;
    let theme = deckcraft_model::theme::builtin_themes()
        .into_iter()
        .find(|t| t.name.eq_ignore_ascii_case(starter.theme))
        .ok_or_else(|| bad("file.new", format!("template `{}` names a missing theme `{}`", starter.name, starter.theme)))?;
    let mut doc = defaults::blank_presentation(size, theme, false);
    for seed in starter.slides {
        let key = Value::String(seed.layout.to_string());
        let layout = find_layout(&doc, &key, 0)
            .ok_or_else(|| bad("file.new", format!("template `{}` uses an unknown layout `{}`", starter.name, seed.layout)))?;
        let mut slide = defaults::new_slide(&mut doc, layout).ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?;
        if !seed.title.is_empty()
            && let Some(sh) = slide.shapes.iter_mut().find(|sh| sh.ph_type().is_some_and(|k| k.is_title()))
        {
            sh.text = Some(text_with_levels(seed.title));
        }
        let mut bodies = seed.bodies.iter();
        for sh in slide.shapes.iter_mut().filter(|sh| sh.ph_type().is_some_and(|k| !k.is_title())) {
            let Some(text) = bodies.next() else { break };
            if !text.is_empty() {
                sh.text = Some(text_with_levels(text));
            }
        }
        if bodies.next().is_some() {
            return Err(bad("file.new", format!("template `{}` has more text than the `{}` layout holds", starter.name, seed.layout)));
        }
        if !seed.notes.is_empty() {
            slide.notes = text_with_levels(seed.notes);
        }
        doc.slides.push(Arc::new(slide));
    }

    if starter.name.eq_ignore_ascii_case("Photo Album") {
        const TEST_PNG: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0xf3, 0xff,
            0x61, 0x00, 0x00, 0x00, 0x19, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xfc, 0xff, 0xff, 0x3f,
            0x03, 0x10, 0x04, 0x82, 0x81, 0x81, 0x81, 0xa1, 0xa1, 0xa1, 0xa1, 0x01, 0x00, 0x31, 0x1e, 0x04,
            0x01, 0x47, 0xa9, 0x86, 0x34, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60,
            0x82,
        ];
        let media_id = doc.add_media("album_sample.png", "image/png", TEST_PNG.to_vec()).ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?;
        for slide in &mut doc.slides {
            let s = Arc::make_mut(slide);
            for sh in &mut s.shapes {
                if sh.ph_type().is_some_and(|k| matches!(k, PhType::Picture | PhType::Obj | PhType::Body)) && sh.text.as_ref().is_none_or(|t| t.is_empty()) {
                    sh.kind = ShapeKind::Picture {
                        fill: PictureFill {
                            media: media_id,
                            crop: [0.0; 4],
                            ..Default::default()
                        },
                    };
                }
            }
        }
    }

    if starter.name.eq_ignore_ascii_case("Interactive Quiz") && doc.slides.len() >= 8 {
        let slide_ids: Vec<deckcraft_model::SlideId> = doc.slides.iter().map(|s| s.id).collect();
        let btn_0 = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let id_1a = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let id_1b = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let btn_2 = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let btn_3 = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let id_4a = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let id_4b = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let btn_5 = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let btn_6 = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);
        let btn_7 = ShapeId(doc.alloc_id().ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?);

        // Slide 0: Start Button
        {
            let s0 = Arc::make_mut(&mut doc.slides[0]);
            s0.shapes.push(Shape {
                id: btn_0,
                name: "Start Quiz Button".into(),
                xfrm: Some(Xfrm::new((size.width - 200.0) / 2.0, size.height * 0.72, 200.0, 48.0)),
                text: Some(TextBody::from_text("Start Quiz →")),
                click: Some(make_link(Action::Slide { slide: slide_ids[1] }, "Begin the Quiz")),
                ..Default::default()
            });
        }
        // Slide 1: Question 1 options (Choice A -> Slide 2 correct, Choice B -> Slide 3 incorrect)
        {
            let s1 = Arc::make_mut(&mut doc.slides[1]);
            s1.shapes.push(Shape {
                id: id_1a,
                name: "Option A Button".into(),
                xfrm: Some(Xfrm::new(80.0, size.height * 0.45, size.width - 160.0, 48.0)),
                text: Some(TextBody::from_text("Option A: Native offline document productivity")),
                click: Some(make_link(Action::Slide { slide: slide_ids[2] }, "Select Option A")),
                ..Default::default()
            });
            s1.shapes.push(Shape {
                id: id_1b,
                name: "Option B Button".into(),
                xfrm: Some(Xfrm::new(80.0, size.height * 0.58, size.width - 160.0, 48.0)),
                text: Some(TextBody::from_text("Option B: Streaming social video reels")),
                click: Some(make_link(Action::Slide { slide: slide_ids[3] }, "Select Option B")),
                ..Default::default()
            });
        }
        // Slide 2: Correct -> Next Question (Slide 4)
        {
            let s2 = Arc::make_mut(&mut doc.slides[2]);
            s2.shapes.push(Shape {
                id: btn_2,
                name: "Next Question Button".into(),
                xfrm: Some(Xfrm::new(80.0, size.height * 0.65, 220.0, 48.0)),
                text: Some(TextBody::from_text("Next Question →")),
                click: Some(make_link(Action::Slide { slide: slide_ids[4] }, "Proceed to Question 2")),
                ..Default::default()
            });
        }
        // Slide 3: Incorrect -> Retry (Slide 1)
        {
            let s3 = Arc::make_mut(&mut doc.slides[3]);
            s3.shapes.push(Shape {
                id: btn_3,
                name: "Retry Question Button".into(),
                xfrm: Some(Xfrm::new(80.0, size.height * 0.65, 220.0, 48.0)),
                text: Some(TextBody::from_text("← Try Again")),
                click: Some(make_link(Action::Slide { slide: slide_ids[1] }, "Return to Question 1")),
                ..Default::default()
            });
        }
        // Slide 4: Question 2 options (Choice A -> Slide 5 correct, Choice B -> Slide 6 incorrect)
        {
            let s4 = Arc::make_mut(&mut doc.slides[4]);
            s4.shapes.push(Shape {
                id: id_4a,
                name: "Option A Button".into(),
                xfrm: Some(Xfrm::new(80.0, size.height * 0.45, size.width - 160.0, 48.0)),
                text: Some(TextBody::from_text("Option A: Standard OOXML DOCX with high fidelity")),
                click: Some(make_link(Action::Slide { slide: slide_ids[5] }, "Select Option A")),
                ..Default::default()
            });
            s4.shapes.push(Shape {
                id: id_4b,
                name: "Option B Button".into(),
                xfrm: Some(Xfrm::new(80.0, size.height * 0.58, size.width - 160.0, 48.0)),
                text: Some(TextBody::from_text("Option B: Unstructured plaintext only")),
                click: Some(make_link(Action::Slide { slide: slide_ids[6] }, "Select Option B")),
                ..Default::default()
            });
        }
        // Slide 5: Correct -> Complete (Slide 7)
        {
            let s5 = Arc::make_mut(&mut doc.slides[5]);
            s5.shapes.push(Shape {
                id: btn_5,
                name: "Finish Quiz Button".into(),
                xfrm: Some(Xfrm::new(80.0, size.height * 0.65, 220.0, 48.0)),
                text: Some(TextBody::from_text("Complete Quiz →")),
                click: Some(make_link(Action::Slide { slide: slide_ids[7] }, "View Summary")),
                ..Default::default()
            });
        }
        // Slide 6: Incorrect -> Retry (Slide 4)
        {
            let s6 = Arc::make_mut(&mut doc.slides[6]);
            s6.shapes.push(Shape {
                id: btn_6,
                name: "Retry Question Button".into(),
                xfrm: Some(Xfrm::new(80.0, size.height * 0.65, 220.0, 48.0)),
                text: Some(TextBody::from_text("← Try Again")),
                click: Some(make_link(Action::Slide { slide: slide_ids[4] }, "Return to Question 2")),
                ..Default::default()
            });
        }
        // Slide 7: Complete -> Restart (Slide 0)
        {
            let s7 = Arc::make_mut(&mut doc.slides[7]);
            s7.shapes.push(Shape {
                id: btn_7,
                name: "Restart Quiz Button".into(),
                xfrm: Some(Xfrm::new((size.width - 200.0) / 2.0, size.height * 0.72, 200.0, 48.0)),
                text: Some(TextBody::from_text("Restart Quiz ↺")),
                click: Some(make_link(Action::Slide { slide: slide_ids[0] }, "Restart Quiz")),
                ..Default::default()
            });
        }
    }

    Ok(doc)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::Session;

    fn body(doc: &Presentation, slide: usize, n: usize) -> (String, u8) {
        let para_slide = &doc.slides[slide];
        let text = para_slide
            .shapes
            .iter()
            .filter(|sh| sh.ph_type().is_some_and(|k| !k.is_title()))
            .nth(n)
            .and_then(|sh| sh.text.as_ref())
            .cloned()
            .unwrap_or_default();
        let level = text.paragraphs.get(1).map(|p| p.level).unwrap_or(0);
        (text.text(), level)
    }

    #[test]
    fn every_starter_opens_on_its_theme_and_is_a_clean_document() {
        let mut s = Session::new();
        for starter in starters() {
            s.execute("file.new", &json!({"template": starter.name.to_lowercase()})).unwrap();
            let st = s.active().unwrap();
            assert_eq!(st.doc.masters[0].theme.name, starter.theme, "{}", starter.name);
            assert_eq!(st.doc.slides.len(), starter.slides.len(), "{}", starter.name);
            assert_eq!(st.doc.slides[0].title(), starter.slides[0].title, "{}", starter.name);
            assert!(!st.is_dirty(), "{}", starter.name);
            assert!(st.history.undo.is_empty(), "{}", starter.name);
            assert_eq!(st.selection.slide, 0);
        }
        let listed = s.execute("file.templates", &json!({})).unwrap();
        assert_eq!(listed["templates"].as_array().map(Vec::len), Some(starters().len()));
    }

    #[test]
    fn briefing_keeps_notes_columns_and_outline_levels() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"template": "Briefing", "size": [800.0, 450.0]})).unwrap();
        let doc = &s.active().unwrap().doc;
        assert_eq!(doc.slide_size.width, 800.0);
        assert_eq!(doc.slides[0].notes_text(), "Open with what changed since last week.");
        let (agenda, level) = body(doc, 1, 0);
        assert!(agenda.contains("One concrete change"), "{agenda}");
        assert_eq!(level, 1);
        let (done, _) = body(doc, 3, 0);
        let (next, _) = body(doc, 3, 1);
        assert!(done.starts_with("Done"), "{done}");
        assert!(next.starts_with("Next"), "{next}");
    }

    #[test]
    fn pitch_comparison_fills_both_columns() {
        let doc = build("Pitch", defaults::WIDE).unwrap();
        assert_eq!(body(&doc, 3, 0).0, "Today");
        assert!(body(&doc, 3, 1).0.contains("buried"));
        assert_eq!(body(&doc, 3, 2).0, "After");
        assert!(body(&doc, 3, 3).0.contains("top"));
    }

    #[test]
    fn search_matches_name_summary_and_theme() {
        assert_eq!(matching("ridge").iter().map(|t| t.name).collect::<Vec<_>>(), vec!["Field Notes"]);
        assert_eq!(matching("nocturne").iter().map(|t| t.name).collect::<Vec<_>>(), vec!["Evening Talk"]);
        assert_eq!(matching("").len(), starters().len());
        assert!(matching("facet").is_empty());
    }

    #[test]
    fn unknown_blank_and_theme_are_refused() {
        let mut s = Session::new();
        let unknown = s.execute("file.new", &json!({"template": "Ion"})).unwrap_err();
        assert!(unknown.to_string().contains("Briefing"), "{unknown}");
        let both = s.execute("file.new", &json!({"template": "Lesson", "theme": "Harbor"})).unwrap_err();
        assert!(both.to_string().contains("not both"), "{both}");
        let blank = s.execute("file.new", &json!({"template": "Lesson", "blank": true})).unwrap_err();
        assert!(blank.to_string().contains("blank"), "{blank}");
        assert!(s.active().is_none());
    }

    #[test]
    fn a_starter_round_trips_through_pptx() {
        let doc = build("Evening Talk", defaults::WIDE).unwrap();
        let bytes = deckcraft_pptx::export(&doc).unwrap();
        let back = deckcraft_pptx::import(&bytes).unwrap();
        assert_eq!(back.slides.len(), doc.slides.len());
        for (before, after) in doc.slides.iter().zip(back.slides.iter()) {
            assert_eq!(after.title(), before.title());
        }
        assert!(back.slides[3].shapes.iter().any(|sh| sh.text.as_ref().is_some_and(|t| t.text().contains("bedroom"))));
    }

    #[test]
    fn interactive_quiz_links_navigate_correctly() {
        use deckcraft_anim::{ShowAction, ShowState};
        let doc = build("Interactive Quiz", defaults::WIDE).unwrap();
        assert_eq!(doc.slides.len(), 8);
        let mut show = ShowState::new(&doc);
        assert_eq!(show.slide, 0);

        // Click Start on Slide 0 -> transitions to Slide 1
        let start_btn = doc.slides[0].shapes.iter().find(|s| s.click.is_some()).unwrap();
        let jump = show.follow_link(&doc, start_btn.click.as_ref().unwrap(), None);
        assert_eq!(show.slide, 1);
        assert!(matches!(jump, deckcraft_anim::LinkJump::Show(ShowAction::Slide(1))));

        // Click Option A (Correct) on Slide 1 -> transitions to Slide 2
        let opt_a = doc.slides[1].shapes.iter().find(|s| s.name.contains("Option A")).unwrap();
        let jump = show.follow_link(&doc, opt_a.click.as_ref().unwrap(), None);
        assert_eq!(show.slide, 2);
        assert!(matches!(jump, deckcraft_anim::LinkJump::Show(ShowAction::Slide(2))));

        // Click Next Question on Slide 2 -> transitions to Slide 4 (Question 2)
        let next_btn = doc.slides[2].shapes.iter().find(|s| s.click.is_some()).unwrap();
        let jump = show.follow_link(&doc, next_btn.click.as_ref().unwrap(), None);
        assert_eq!(show.slide, 4);
        assert!(matches!(jump, deckcraft_anim::LinkJump::Show(ShowAction::Slide(4))));
    }

    #[test]
    fn photo_album_has_picture_placeholder_and_replaces_cleanly() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"template": "photo album"})).unwrap();
        let doc = &s.active().unwrap().doc;
        assert_eq!(doc.slides.len(), 4);
        let pic_shape = doc.slides[1].shapes.iter().find(|sh| matches!(sh.kind, ShapeKind::Picture { .. })).unwrap();
        assert!(pic_shape.xfrm.is_some());
        let orig_xfrm = pic_shape.xfrm.unwrap();
        let orig_media = match &pic_shape.kind {
            ShapeKind::Picture { fill } => fill.media,
            _ => unreachable!(),
        };

        // Select the picture shape and replace the picture
        s.execute("selection.select", &json!({"slide": 1, "shapes": [pic_shape.id.0]})).unwrap();
        const REPLACEMENT_PNG: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0xf3, 0xff,
            0x61, 0x00, 0x00, 0x00, 0x19, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xfc, 0xff, 0xff, 0x3f,
            0x03, 0x10, 0x04, 0x82, 0x81, 0x81, 0x81, 0xa1, 0xa1, 0xa1, 0xa1, 0x01, 0x00, 0x31, 0x1e, 0x04,
            0x01, 0x47, 0xa9, 0x86, 0x34, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60,
            0x82,
        ];
        use crate::cmd::base64_encode;
        s.execute("picture.change", &json!({"data": base64_encode(REPLACEMENT_PNG), "name": "replaced.png"})).unwrap();

        let updated_doc = &s.active().unwrap().doc;
        let updated_pic = updated_doc.slides[1].shapes.iter().find(|sh| sh.id == pic_shape.id).unwrap();
        let new_xfrm = updated_pic.xfrm.unwrap();
        assert_eq!(orig_xfrm.x, new_xfrm.x);
        assert_eq!(orig_xfrm.y, new_xfrm.y);
        assert_eq!(orig_xfrm.w, new_xfrm.w);
        assert_eq!(orig_xfrm.h, new_xfrm.h);
        match &updated_pic.kind {
            ShapeKind::Picture { fill } => {
                assert_ne!(fill.media, orig_media, "Media ID should be replaced");
            }
            _ => panic!("Expected Picture shape"),
        }
    }
}
