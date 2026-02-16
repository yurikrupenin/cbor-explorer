use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub fg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border_focused: Color,
    pub border_unfocused: Color,
    pub header_fg: Color,
    pub depth_colors: Vec<Color>,
    pub byte_colors: ByteColors,
    pub popup_border: Color,
    pub popup_bg: Color,
}

#[derive(Debug, Clone)]
pub struct ByteColors {
    pub null: Color,
    pub ascii_printable: Color,
    pub ascii_whitespace: Color,
    pub ascii_other: Color,
    pub non_ascii: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::tokyo_night()
    }
}

impl Theme {
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(169, 177, 214),             // Text
            selection_bg: Color::Rgb(51, 70, 124),     // Selection Background
            selection_fg: Color::Rgb(192, 202, 245),   // Selection Foreground
            border_focused: Color::Rgb(122, 162, 247), // Blue
            border_unfocused: Color::Rgb(86, 95, 137), // Comment/Darker
            header_fg: Color::Rgb(122, 162, 247),
            depth_colors: vec![
                Color::Rgb(255, 215, 0),  // Gold - selected item (depth 0)
                Color::Rgb(255, 165, 0),  // Orange - parent
                Color::Rgb(255, 99, 71),  // Tomato - grandparent
                Color::Rgb(199, 21, 133), // MediumVioletRed
                Color::Rgb(138, 43, 226), // BlueViolet
                Color::Rgb(65, 105, 225), // RoyalBlue
                Color::Rgb(30, 144, 255), // DodgerBlue
                Color::Rgb(0, 191, 255),  // DeepSkyBlue - root
            ],
            byte_colors: ByteColors {
                null: Color::DarkGray,
                ascii_printable: Color::Blue,
                ascii_whitespace: Color::Rgb(67, 205, 128),
                ascii_other: Color::Indexed(162),
                non_ascii: Color::Red,
            },
            popup_border: Color::Rgb(122, 162, 247),
            popup_bg: Color::Reset,
        }
    }

    pub fn tokyo_night_storm() -> Self {
        Self {
            name: "Tokyo Night Storm".to_string(),
            bg: Color::Rgb(36, 40, 59),
            fg: Color::Rgb(192, 202, 245),
            selection_bg: Color::Rgb(65, 72, 104),
            selection_fg: Color::Rgb(192, 202, 245),
            border_focused: Color::Rgb(122, 162, 247), // Blue
            border_unfocused: Color::Rgb(86, 95, 137), // Comment
            header_fg: Color::Rgb(122, 162, 247),
            depth_colors: vec![
                Color::Rgb(255, 158, 100), // Orange
                Color::Rgb(247, 118, 142), // Red
                Color::Rgb(187, 154, 247), // Magenta
                Color::Rgb(122, 162, 247), // Blue
                Color::Rgb(125, 207, 255), // Cyan
                Color::Rgb(158, 206, 106), // Green
                Color::Rgb(224, 175, 104), // Yellow
                Color::Rgb(255, 158, 100), // Orange
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(86, 95, 137),
                ascii_printable: Color::Rgb(122, 162, 247),
                ascii_whitespace: Color::Rgb(158, 206, 106),
                ascii_other: Color::Rgb(224, 175, 104),
                non_ascii: Color::Rgb(247, 118, 142),
            },
            popup_border: Color::Rgb(122, 162, 247),
            popup_bg: Color::Rgb(36, 40, 59),
        }
    }

    pub fn tokyo_night_light() -> Self {
        Self {
            name: "Tokyo Night Light".to_string(),
            bg: Color::Rgb(230, 231, 237),
            fg: Color::Rgb(52, 59, 88),
            selection_bg: Color::Rgb(208, 213, 227),
            selection_fg: Color::Rgb(52, 59, 88),
            border_focused: Color::Rgb(41, 89, 170), // Blue
            border_unfocused: Color::Rgb(154, 165, 206), // Comment
            header_fg: Color::Rgb(41, 89, 170),
            depth_colors: vec![
                Color::Rgb(143, 94, 21), // Yellow/Bronze
                Color::Rgb(140, 67, 81), // Red
                Color::Rgb(90, 62, 142), // Magenta
                Color::Rgb(41, 89, 170), // Blue
                Color::Rgb(0, 108, 134), // Cyan
                Color::Rgb(56, 95, 13),  // Green
                Color::Rgb(143, 94, 21), // Yellow
                Color::Rgb(140, 67, 81), // Red
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(154, 165, 206),
                ascii_printable: Color::Rgb(41, 89, 170),
                ascii_whitespace: Color::Rgb(56, 95, 13),
                ascii_other: Color::Rgb(143, 94, 21),
                non_ascii: Color::Rgb(140, 67, 81),
            },
            popup_border: Color::Rgb(41, 89, 170),
            popup_bg: Color::Rgb(230, 231, 237),
        }
    }

    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            selection_bg: Color::Rgb(68, 71, 90),
            selection_fg: Color::Rgb(255, 255, 255),
            border_focused: Color::Rgb(189, 147, 249), // Purple
            border_unfocused: Color::Rgb(98, 114, 164), // Comment
            header_fg: Color::Rgb(80, 250, 123),       // Green
            depth_colors: vec![
                Color::Rgb(255, 121, 198), // Pink
                Color::Rgb(189, 147, 249), // Purple
                Color::Rgb(139, 233, 253), // Cyan
                Color::Rgb(80, 250, 123),  // Green
                Color::Rgb(241, 250, 140), // Yellow
                Color::Rgb(255, 184, 108), // Orange
                Color::Rgb(255, 85, 85),   // Red
                Color::Rgb(98, 114, 164),  // Comment
            ],
            byte_colors: ByteColors {
                null: Color::DarkGray,
                ascii_printable: Color::Rgb(139, 233, 253),
                ascii_whitespace: Color::Rgb(80, 250, 123),
                ascii_other: Color::Rgb(255, 184, 108),
                non_ascii: Color::Rgb(255, 85, 85),
            },
            popup_border: Color::Rgb(189, 147, 249),
            popup_bg: Color::Rgb(40, 42, 54),
        }
    }

    pub fn solarized() -> Self {
        Self {
            name: "Solarized".to_string(),
            bg: Color::Rgb(0, 43, 54),                  // Base03
            fg: Color::Rgb(131, 148, 150),              // Base0
            selection_bg: Color::Rgb(7, 54, 66),        // Base02
            selection_fg: Color::Rgb(147, 161, 161),    // Base1
            border_focused: Color::Rgb(38, 139, 210),   // Blue
            border_unfocused: Color::Rgb(88, 110, 117), // Base01
            header_fg: Color::Rgb(133, 153, 0),         // Green
            depth_colors: vec![
                Color::Rgb(181, 137, 0),   // Yellow
                Color::Rgb(203, 75, 22),   // Orange
                Color::Rgb(220, 50, 47),   // Red
                Color::Rgb(211, 54, 130),  // Magenta
                Color::Rgb(108, 113, 196), // Violet
                Color::Rgb(38, 139, 210),  // Blue
                Color::Rgb(42, 161, 152),  // Cyan
                Color::Rgb(133, 153, 0),   // Green
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(88, 110, 117), // Base01
                ascii_printable: Color::Rgb(38, 139, 210),
                ascii_whitespace: Color::Rgb(133, 153, 0),
                ascii_other: Color::Rgb(203, 75, 22),
                non_ascii: Color::Rgb(220, 50, 47),
            },
            popup_border: Color::Rgb(38, 139, 210),
            popup_bg: Color::Rgb(0, 43, 54),
        }
    }

    pub fn monokai() -> Self {
        Self {
            name: "Monokai".to_string(),
            bg: Color::Rgb(39, 40, 34),
            fg: Color::Rgb(248, 248, 242),
            selection_bg: Color::Rgb(73, 72, 62),
            selection_fg: Color::Rgb(248, 248, 242),
            border_focused: Color::Rgb(166, 226, 46), // Green
            border_unfocused: Color::Rgb(117, 113, 94), // Grey
            header_fg: Color::Rgb(102, 217, 239),     // Blue
            depth_colors: vec![
                Color::Rgb(253, 151, 31),  // Orange
                Color::Rgb(166, 226, 46),  // Green
                Color::Rgb(102, 217, 239), // Blue
                Color::Rgb(174, 129, 255), // Purple
                Color::Rgb(249, 38, 114),  // Pink
                Color::Rgb(253, 151, 31),  // Orange
                Color::Rgb(166, 226, 46),  // Green
                Color::Rgb(102, 217, 239), // Blue
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(117, 113, 94),
                ascii_printable: Color::Rgb(166, 226, 46),
                ascii_whitespace: Color::Rgb(249, 38, 114),
                ascii_other: Color::Rgb(174, 129, 255),
                non_ascii: Color::Rgb(253, 151, 31),
            },
            popup_border: Color::Rgb(166, 226, 46),
            popup_bg: Color::Rgb(39, 40, 34),
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(236, 239, 244),
            selection_bg: Color::Rgb(67, 76, 94),
            selection_fg: Color::Rgb(236, 239, 244),
            border_focused: Color::Rgb(136, 192, 208), // Cyan
            border_unfocused: Color::Rgb(76, 86, 106),
            header_fg: Color::Rgb(143, 188, 187), // Teal
            depth_colors: vec![
                Color::Rgb(191, 97, 106),  // Red
                Color::Rgb(208, 135, 112), // Orange
                Color::Rgb(235, 203, 139), // Yellow
                Color::Rgb(163, 190, 140), // Green
                Color::Rgb(180, 142, 173), // Purple
                Color::Rgb(136, 192, 208), // Cyan
                Color::Rgb(129, 161, 193), // Blue
                Color::Rgb(94, 129, 172),  // Blue
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(76, 86, 106),
                ascii_printable: Color::Rgb(163, 190, 140),
                ascii_whitespace: Color::Rgb(235, 203, 139),
                ascii_other: Color::Rgb(208, 135, 112),
                non_ascii: Color::Rgb(191, 97, 106),
            },
            popup_border: Color::Rgb(136, 192, 208),
            popup_bg: Color::Rgb(46, 52, 64),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            name: "Gruvbox".to_string(),
            bg: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(235, 219, 178),
            selection_bg: Color::Rgb(60, 56, 54),
            selection_fg: Color::Rgb(251, 241, 199),
            border_focused: Color::Rgb(254, 128, 25), // Orange
            border_unfocused: Color::Rgb(146, 131, 116),
            header_fg: Color::Rgb(184, 187, 38), // Green
            depth_colors: vec![
                Color::Rgb(251, 73, 52),   // Red
                Color::Rgb(254, 128, 25),  // Orange
                Color::Rgb(250, 189, 47),  // Yellow
                Color::Rgb(184, 187, 38),  // Green
                Color::Rgb(131, 165, 152), // Blue
                Color::Rgb(211, 134, 155), // Purple
                Color::Rgb(142, 192, 124), // Aqua
                Color::Rgb(254, 128, 25),  // Orange
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(146, 131, 116),
                ascii_printable: Color::Rgb(184, 187, 38),
                ascii_whitespace: Color::Rgb(250, 189, 47),
                ascii_other: Color::Rgb(131, 165, 152),
                non_ascii: Color::Rgb(251, 73, 52),
            },
            popup_border: Color::Rgb(254, 128, 25),
            popup_bg: Color::Rgb(40, 40, 40),
        }
    }

    pub fn one_dark() -> Self {
        Self {
            name: "One Dark".to_string(),
            bg: Color::Rgb(40, 44, 52),
            fg: Color::Rgb(171, 178, 191),
            selection_bg: Color::Rgb(62, 68, 81),
            selection_fg: Color::Rgb(220, 223, 228),
            border_focused: Color::Rgb(97, 175, 239), // Blue
            border_unfocused: Color::Rgb(92, 99, 112),
            header_fg: Color::Rgb(152, 195, 121), // Green
            depth_colors: vec![
                Color::Rgb(224, 108, 117), // Red
                Color::Rgb(209, 154, 102), // Orange
                Color::Rgb(229, 192, 123), // Yellow
                Color::Rgb(152, 195, 121), // Green
                Color::Rgb(86, 182, 194),  // Cyan
                Color::Rgb(97, 175, 239),  // Blue
                Color::Rgb(198, 120, 221), // Purple
                Color::Rgb(224, 108, 117), // Red
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(92, 99, 112),
                ascii_printable: Color::Rgb(152, 195, 121),
                ascii_whitespace: Color::Rgb(229, 192, 123),
                ascii_other: Color::Rgb(86, 182, 194),
                non_ascii: Color::Rgb(224, 108, 117),
            },
            popup_border: Color::Rgb(97, 175, 239),
            popup_bg: Color::Rgb(40, 44, 52),
        }
    }

    pub fn catppuccin() -> Self {
        Self {
            name: "Catppuccin".to_string(),
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            selection_bg: Color::Rgb(50, 50, 70), // Reduced brightness
            selection_fg: Color::Rgb(205, 214, 244),
            border_focused: Color::Rgb(137, 180, 250), // Blue
            border_unfocused: Color::Rgb(88, 91, 112),
            header_fg: Color::Rgb(166, 227, 161), // Green
            depth_colors: vec![
                Color::Rgb(243, 139, 168), // Red
                Color::Rgb(250, 179, 135), // Peach
                Color::Rgb(249, 226, 175), // Yellow
                Color::Rgb(166, 227, 161), // Green
                Color::Rgb(137, 220, 235), // Sky
                Color::Rgb(137, 180, 250), // Blue
                Color::Rgb(203, 166, 247), // Mauve
                Color::Rgb(245, 194, 231), // Pink
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(88, 91, 112),
                ascii_printable: Color::Rgb(166, 227, 161),
                ascii_whitespace: Color::Rgb(249, 226, 175),
                ascii_other: Color::Rgb(137, 220, 235),
                non_ascii: Color::Rgb(243, 139, 168),
            },
            popup_border: Color::Rgb(137, 180, 250),
            popup_bg: Color::Rgb(30, 30, 46),
        }
    }

    pub fn rose_pine() -> Self {
        Self {
            name: "Rosé Pine".to_string(),
            bg: Color::Rgb(25, 23, 36),
            fg: Color::Rgb(224, 222, 244),
            selection_bg: Color::Rgb(64, 61, 82), // Highlight Med
            selection_fg: Color::Rgb(224, 222, 244),
            border_focused: Color::Rgb(196, 167, 231), // Iris
            border_unfocused: Color::Rgb(110, 106, 134), // Muted
            header_fg: Color::Rgb(49, 116, 143),       // Pine
            depth_colors: vec![
                Color::Rgb(246, 193, 119), // Gold
                Color::Rgb(235, 188, 186), // Rose
                Color::Rgb(235, 111, 146), // Love
                Color::Rgb(196, 167, 231), // Iris
                Color::Rgb(156, 207, 216), // Foam
                Color::Rgb(49, 116, 143),  // Pine
                Color::Rgb(246, 193, 119), // Gold
                Color::Rgb(235, 188, 186), // Rose
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(110, 106, 134),             // Muted
                ascii_printable: Color::Rgb(224, 222, 244),  // Text
                ascii_whitespace: Color::Rgb(144, 140, 170), // Subtle
                ascii_other: Color::Rgb(246, 193, 119),      // Gold
                non_ascii: Color::Rgb(235, 111, 146),        // Love
            },
            popup_border: Color::Rgb(196, 167, 231), // Iris
            popup_bg: Color::Rgb(25, 23, 36),
        }
    }

    pub fn rose_pine_moon() -> Self {
        Self {
            name: "Rosé Pine Moon".to_string(),
            bg: Color::Rgb(35, 33, 54),
            fg: Color::Rgb(224, 222, 244),
            selection_bg: Color::Rgb(68, 65, 90), // Highlight Med
            selection_fg: Color::Rgb(224, 222, 244),
            border_focused: Color::Rgb(196, 167, 231), // Iris
            border_unfocused: Color::Rgb(110, 106, 134), // Muted
            header_fg: Color::Rgb(62, 143, 176),       // Pine
            depth_colors: vec![
                Color::Rgb(246, 193, 119), // Gold
                Color::Rgb(234, 154, 151), // Rose
                Color::Rgb(235, 111, 146), // Love
                Color::Rgb(196, 167, 231), // Iris
                Color::Rgb(156, 207, 216), // Foam
                Color::Rgb(62, 143, 176),  // Pine
                Color::Rgb(246, 193, 119), // Gold
                Color::Rgb(234, 154, 151), // Rose
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(110, 106, 134),
                ascii_printable: Color::Rgb(224, 222, 244),
                ascii_whitespace: Color::Rgb(144, 140, 170),
                ascii_other: Color::Rgb(246, 193, 119),
                non_ascii: Color::Rgb(235, 111, 146),
            },
            popup_border: Color::Rgb(196, 167, 231),
            popup_bg: Color::Rgb(35, 33, 54),
        }
    }

    pub fn rose_pine_dawn() -> Self {
        Self {
            name: "Rosé Pine Dawn".to_string(),
            bg: Color::Rgb(250, 244, 237),
            fg: Color::Rgb(87, 82, 121),
            selection_bg: Color::Rgb(223, 218, 217), // Highlight Med
            selection_fg: Color::Rgb(87, 82, 121),
            border_focused: Color::Rgb(144, 122, 169), // Iris
            border_unfocused: Color::Rgb(152, 147, 165), // Muted
            header_fg: Color::Rgb(40, 105, 131),       // Pine
            depth_colors: vec![
                Color::Rgb(234, 157, 52),  // Gold
                Color::Rgb(215, 130, 126), // Rose
                Color::Rgb(180, 99, 122),  // Love
                Color::Rgb(144, 122, 169), // Iris
                Color::Rgb(86, 148, 159),  // Foam
                Color::Rgb(40, 105, 131),  // Pine
                Color::Rgb(234, 157, 52),  // Gold
                Color::Rgb(215, 130, 126), // Rose
            ],
            byte_colors: ByteColors {
                null: Color::Rgb(152, 147, 165),
                ascii_printable: Color::Rgb(87, 82, 121),
                ascii_whitespace: Color::Rgb(121, 117, 147),
                ascii_other: Color::Rgb(234, 157, 52),
                non_ascii: Color::Rgb(180, 99, 122),
            },
            popup_border: Color::Rgb(144, 122, 169),
            popup_bg: Color::Rgb(250, 244, 237),
        }
    }

    pub fn get_depth_color(&self, depth: usize) -> Color {
        let index = 7 - (depth % 8);
        if index < self.depth_colors.len() {
            self.depth_colors[index]
        } else {
            self.depth_colors[0] // Fallback
        }
    }
}
