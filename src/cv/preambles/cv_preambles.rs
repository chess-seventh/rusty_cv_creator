use latex::PreambleElement;

pub fn build_preamble() -> Vec<PreambleElement> {
    let mut packages = build_packages();
    let mut colors = build_colors();
    let mut my_infos = build_my_info();

    // TODO: make my position dynamic with a param and move to another function
    let my_position =
        PreambleElement::UserDefined("\\newcommand\\myposition{BLANKPOSITION}".to_string());

    let mut preambles = vec![my_position];

    preambles.append(&mut packages);
    preambles.append(&mut colors);
    preambles.append(&mut my_infos);

    preambles
}

fn build_packages() -> Vec<PreambleElement> {
    let apple_emojis = PreambleElement::UsePackage {
        package: "coloremoji".to_string(),
        argument: Some("apple".to_string()),
    };

    let geometry = PreambleElement::UserDefined(
        r#"\geometry{left=1.5cm, top=0.7cm, right=1.5cm, bottom=0.7cm, footskip=.5cm}"#.to_string(),
    );

    let fonts = PreambleElement::UserDefined(r#"\fontdir[./]"#.to_string());

    let footnote = PreambleElement::UserDefined(r#"\newcommand\blfootnote[1]{\begingroup\renewcommand\thefootnote{}\footnote{#1}\addtocounter{footnote}{-1}\endgroup}"#.to_string());

    let footer = PreambleElement::UserDefined(r#"\makecvfooter{francesco@piva.online}{~~~·~~~\textbf{Francesco Piva}~~~·~~~}{(+41) 79 830 02 70}"#.to_string());

    let socials = PreambleElement::UserDefined(
        r#"\renewcommand{\acvHeaderSocialSep}{\quad\textbar\quad}"#.to_string(),
    );

    vec![apple_emojis, geometry, fonts, socials, footnote, footer]
}

fn build_colors(
    color_red: Option<String>,
    color_line: Option<String>,
    color_gray: Option<String>,
) -> Vec<PreambleElement> {
    let colors = PreambleElement::UserDefined(r#"\colorlet{awesome}{awesome-red}"#.to_string());

    // TODO: make color dynamic
    let my_red = match color_red {
        Some(color) => {
            let formatted_red = format!("\\definecolor{{my-red}}{{HTML}}{{{:}}}", color);
            let my_red = PreambleElement::UserDefined(formatted_red.to_string());
            my_red
        }
        None => {
            let my_color = "d58787".to_string();
            let formatted_red = format!("\\definecolor{{my-red}}{{HTML}}{{{:}}}", my_color);
            let my_red = PreambleElement::UserDefined(formatted_red.to_string());
            my_red
        }
    };

    // TODO: make color dynamic
    let line_color =
        PreambleElement::UserDefined(r#"\definecolor{line-color}{HTML}{fde2a0}"#.to_string());

    // TODO: make color dynamic
    let light_gray =
        PreambleElement::UserDefined(r#"\definecolor{light-gray}{HTML}{e6e6e6}"#.to_string());

    let hll = PreambleElement::UserDefined(r#"\newcommand{\hll}[1]{\noindent\colorbox{light-gray}{\parbox{17.5cm}{\textcolor{gray}{#1}}}}"#.to_string());

    vec![colors, my_red, line_color, light_gray, hll]
}

fn build_my_info() -> Vec<PreambleElement> {
    vec![
        PreambleElement::UserDefined(
            r#"\name{Francesco}{\\textcolor{awesome-red}{Piva}}"#.to_string(),
        ),
        PreambleElement::UserDefined(r#"\address{🌍 Switzerland}"#.to_string()),
        PreambleElement::UserDefined(r#"\permis{Swiss, 37 years old}"#.to_string()),
        PreambleElement::UserDefined(r#"\mobile{(+41) 79 830 02 70}"#.to_string()),
        PreambleElement::UserDefined(r#"\email{francesco@piva.online}"#.to_string()),
    ]
}
