//! The Craft App Suite: the seventeen apps the launcher knows about.
//!
//! The list is data so a later version can fetch it from an API; [`SUITE`] is the built-in copy.
//!
//! Text here is English, the source the UI translates from: each `tagline` has a catalog row
//! keyed by the app id (`tr_ctx(app.id, app.tagline)` in `ui-egui`).

/// Who makes an app. Official apps live under [`GITHUB_OWNER`]; a community app is maintained
/// outside ArtCraft and is labelled as such wherever it is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppKind {
    Official,
    Community,
}

/// One app in the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftApp {
    /// Machine name: repo, binary, package and asset stem (`photocraft`).
    pub id: &'static str,
    /// Display name (`PhotoCraft`).
    pub name: &'static str,
    /// What it's for, in a few words (English; the UI translates it by app id).
    pub tagline: &'static str,
    /// Brand colour (sRGB), for surfaces: getartcraft.com's `--app`.
    pub color: [u8; 3],
    /// The brand colour for text on a light background (the site's light `--app-ink`).
    pub ink_light: [u8; 3],
    /// The brand colour for text on a dark background (the site's dark `--app-ink`).
    pub ink_dark: [u8; 3],
    /// GitHub `owner/repo`.
    pub repo: &'static str,
    /// Older names its release assets may carry (PdfCraft shipped as PrintCraft until 0.2.1).
    pub former_ids: &'static [&'static str],
    /// Older display names its installers registered (PdfCraft's MSI said PrintCraft until 0.2.1).
    pub former_names: &'static [&'static str],
    pub kind: AppKind,
    /// Who publishes it, as the card's byline shows it (`ArtCraft`, `Tangency Labs`).
    pub publisher: &'static str,
    /// The publisher its Windows installer names, when it isn't ArtCraft's (`Learning Machines LLC`).
    pub msi_publisher: Option<&'static str>,
    /// The bundle id when it isn't `ai.storyteller.<id>` (a community app brings its own).
    pub bundle: Option<&'static str>,
    /// Older bundle ids its Flatpak shipped under (SolidCraft was `ai.storyteller.solidcraft`).
    pub former_bundles: &'static [&'static str],
    /// Its website, when it isn't a page on getartcraft.com (a community app has its own).
    pub site: Option<&'static str>,
    /// The Apple Developer team whose Developer ID signature its macOS build must carry. `None`
    /// while those builds carry no signature the launcher can check: it then opens a copy that's
    /// there and links to the releases, but installs nothing.
    pub mac_team: Option<&'static str>,
}

impl CraftApp {
    /// `ai.storyteller.<id>`: macOS bundle id, Flatpak id and Linux desktop-file id.
    pub fn bundle_id(&self) -> String {
        self.bundle.map_or_else(|| format!("ai.storyteller.{}", self.id), str::to_string)
    }

    pub fn releases_url(&self) -> String {
        "https://tzlabs.in".to_string()
    }

    pub fn repo_url(&self) -> String {
        "https://tzlabs.in".to_string()
    }

    /// The app's website.
    pub fn website(&self) -> String {
        "https://tzlabs.in".to_string()
    }

    /// Whether the launcher installs this app from a `kind` package.
    pub fn installs(&self, kind: crate::PackageKind) -> bool {
        kind != crate::PackageKind::Dmg || self.mac_team.is_some()
    }

    /// Bundle ids this app may be installed under, current first.
    pub fn bundle_ids(&self) -> impl Iterator<Item = String> + '_ {
        std::iter::once(self.bundle_id()).chain(self.former_bundles.iter().map(|b| b.to_string()))
    }

    /// Asset stems this app's releases may use, current first.
    pub fn stems(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.id).chain(self.former_ids.iter().copied())
    }

    /// Display names this app's installers may have registered, current first.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.name).chain(self.former_names.iter().copied())
    }

    pub fn by_id(id: &str) -> Option<&'static CraftApp> {
        SUITE.iter().find(|a| a.id == id)
    }
}

const fn app(
    id: &'static str,
    name: &'static str,
    tagline: &'static str,
    [color, ink_light, ink_dark]: [[u8; 3]; 3],
    repo: &'static str,
) -> CraftApp {
    CraftApp {
        id,
        name,
        tagline,
        color,
        ink_light,
        ink_dark,
        repo,
        former_ids: &[],
        former_names: &[],
        kind: AppKind::Official,
        publisher: "TZWorkQuint",
        msi_publisher: None,
        bundle: None,
        former_bundles: &[],
        site: None,
        mac_team: Some(ARTCRAFT_TEAM_ID),
    }
}

/// The Craft App Suite, in display order matching getartcraft.com, then the community apps:
pub const SUITE: [CraftApp; 5] = [
    CraftApp {
        former_ids: &["pdfcraft", "printcraft"],
        former_names: &["PdfCraft", "PrintCraft"],
        publisher: "TZWorkQuint",
        ..app(
            "tz-pdflex",
            "TZ PDFlex",
            "Reading, organizing and protecting PDFs",
            [[0xe8, 0x20, 0x4f], [0xb7, 0x02, 0x28], [0xf2, 0x3a, 0x62]],
            "storytold/pdfcraft",
        )
    },
    CraftApp {
        former_ids: &["wordcraft"],
        former_names: &["WordCraft"],
        publisher: "TZWorkQuint",
        ..app(
            "tz-wordloom",
            "TZ Wordloom",
            "Writing and document design",
            [[0x3f, 0x5f, 0xe0], [0x2a, 0x44, 0xb8], [0x8e, 0xa2, 0xff]],
            "storytold/wordcraft",
        )
    },
    CraftApp {
        former_ids: &["gridcraft"],
        former_names: &["GridCraft"],
        publisher: "TZWorkQuint",
        ..app(
            "tz-cellforge",
            "TZ CellForge",
            "Spreadsheets and data",
            [[0x1f, 0x9d, 0x55], [0x12, 0x70, 0x3a], [0x4f, 0xd6, 0x8a]],
            "storytold/gridcraft",
        )
    },
    CraftApp {
        former_ids: &["cadcraft"],
        former_names: &["CADCraft"],
        publisher: "TZWorkQuint",
        ..app(
            "tz-draftline",
            "TZ Draftline",
            "Computer-aided design and drafting",
            [[0xd9, 0x77, 0x06], [0xb4, 0x53, 0x09], [0xfb, 0xbf, 0x24]],
            "storytold/cadcraft",
        )
    },
    CraftApp {
        former_ids: &["deckcraft"],
        former_names: &["DeckCraft"],
        publisher: "TZWorkQuint",
        ..app(
            "tz-slidespark",
            "TZ SlideSpark",
            "Presentations and slide shows",
            [[0x79, 0x50, 0xf2], [0x5f, 0x3d, 0xc4], [0x97, 0x75, 0xfa]],
            "storytold/deckcraft",
        )
    },
];

/// The Apple Developer team that signs every official Craft App and ArtCraft Launcher (Learning
/// Machines LLC): the macOS side of the Windows publisher check.
pub const ARTCRAFT_TEAM_ID: &str = "DJ6XS33FX8";

/// TZLabs / TZWorkQuint website.
pub const WEBSITE_URL: &str = "https://tzlabs.in";
/// The GitHub account every official Craft App's repository lives under. The release feed can
/// only move an app within it; a community app installs from its own repo, pinned in [`SUITE`].
pub const GITHUB_OWNER: &str = "storytold";
/// ArtCraft on GitHub: every Craft App's source.
pub const GITHUB_URL: &str = "https://github.com/storytold";
/// ArtCraft's social accounts: (service, URL). Shown as small icons, less prominent than Discord.
pub const SOCIAL: [(&str, &str); 5] = [
    ("X", "https://x.com/get_artcraft"),
    ("Instagram", "https://www.instagram.com/officialartcraftstudios"),
    ("LinkedIn", "https://www.linkedin.com/company/artcraft-ai"),
    ("YouTube", "https://www.youtube.com/@OfficialArtcraftStudios"),
    ("TikTok", "https://www.tiktok.com/@artcraft.studios"),
];

pub const DISCORD_URL: &str = "https://tzlabs.in";
pub const DISCORD_APP_URL: &str = "https://tzlabs.in";

/// ArtCraft Launcher itself, described like a Craft App so it can update itself with the same
/// machinery (asset names `artcraft-launcher-<ver>-…`, bundle id `ai.storyteller.artcraft-launcher`,
/// `ArtCraft Launcher.app`). Not part of [`SUITE`].
pub const LAUNCHER: CraftApp = app(
    "tz-workquint",
    "TZWorkQuint",
    "Productivity Suite Dashboard",
    [[0x25, 0x63, 0xeb], [0x1d, 0x4e, 0xd8], [0x60, 0xa5, 0xfa]],
    "storytold/craft-launcher",
);

/// The launcher's own repo, checked so it can say when a newer launcher is out.
pub const LAUNCHER_REPO: &str = "storytold/craft-launcher";
/// The launcher's own asset stem.
pub const LAUNCHER_ID: &str = "tz-workquint";

/// A Craft App's 128 px icon (PNG), as copied from its repo into `assets/app-icons`: the UI
/// draws it, and the Linux install puts it in the desktop menu.
pub fn app_icon_png(id: &str) -> Option<&'static [u8]> {
    Some(match id {
        "tz-pdflex" | "pdfcraft" => include_bytes!("../../../assets/app-icons/tz-pdflex.png"),
        "tz-wordloom" | "wordcraft" => include_bytes!("../../../assets/app-icons/tz-wordloom.png"),
        "tz-cellforge" | "gridcraft" => include_bytes!("../../../assets/app-icons/tz-cellforge.png"),
        "tz-draftline" | "cadcraft" => include_bytes!("../../../assets/app-icons/tz-draftline.png"),
        "tz-slidespark" | "deckcraft" => include_bytes!("../../../assets/app-icons/tz-slidespark.png"),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_app_has_its_icon() {
        for app in &SUITE {
            let png = app_icon_png(app.id).unwrap_or_else(|| panic!("no icon for {}", app.id));
            assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "{} icon isn't a PNG", app.id);
        }
        assert!(app_icon_png("notacraft").is_none());
    }

    /// Every catalog entry must have a distinct lookup key.
    #[test]
    fn suite_has_five_apps_with_unique_ids() {
        assert_eq!(SUITE.len(), 5);
        let mut ids: Vec<_> = SUITE.iter().map(|a| a.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), SUITE.len());
    }

    #[test]
    fn pdfcraft_also_answers_to_printcraft() {
        let pdf = CraftApp::by_id("tz-pdflex").unwrap();
        assert_eq!(pdf.stems().collect::<Vec<_>>(), ["tz-pdflex", "pdfcraft", "printcraft"]);
        assert_eq!(pdf.names().collect::<Vec<_>>(), ["TZ PDFlex", "PdfCraft", "PrintCraft"]);
    }
}
