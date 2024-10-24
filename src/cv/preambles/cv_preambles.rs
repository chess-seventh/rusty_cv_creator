use latex::PreambleElement;

pub fn build_preamble(
    my_email: Option<String>,
    my_name: Option<String>,
    my_phone: Option<String>,
    my_country: Option<String>,
    my_position: String,
) -> Vec<PreambleElement> {
    let color_red = None;
    let color_gray = None;
    let color_line = None;

    let mut packages = build_packages();
    let mut colors = build_colors(color_red, color_line, color_gray);
    let mut my_infos = build_my_info(my_email, my_name, my_phone, my_country);

    let my_position = PreambleElement::UserDefined(
        format!("\\newcommand\\myposition{{{:}}}", my_position).to_string(),
    );

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

    let socials = PreambleElement::UserDefined(
        r#"\renewcommand{\acvHeaderSocialSep}{\quad\textbar\quad}"#.to_string(),
    );

    vec![apple_emojis, geometry, fonts, socials, footnote]
}

fn build_colors(
    color_red: Option<String>,
    color_line: Option<String>,
    color_gray: Option<String>,
) -> Vec<PreambleElement> {
    let colors = PreambleElement::UserDefined(r#"\colorlet{awesome}{awesome-red}"#.to_string());

    // TODO: make color dynamic
    let my_red = match color_red {
        Some(color) => PreambleElement::UserDefined(
            format!("\\definecolor{{my-red}}{{HTML}}{{{:}}}", color).to_string(),
        ),
        None => PreambleElement::UserDefined(r#""#.to_string()),
    };

    let line_color = match color_line {
        Some(color) => PreambleElement::UserDefined(
            format!("\\definecolor{{line-color}}{{HTML}}{{{:}}}", color).to_string(),
        ),
        None => PreambleElement::UserDefined(r#""#.to_string()),
    };

    let light_gray = match color_gray {
        Some(color) => PreambleElement::UserDefined(
            format!("\\definecolor{{light-gray}}{{HTML}}{{{:}}}", color).to_string(),
        ),
        None => {
            PreambleElement::UserDefined(r#"\definecolor{light-gray}{HTML}{e6e6e6}"#.to_string())
        }
    };

    let hll = PreambleElement::UserDefined(r#"\newcommand{\hll}[1]{\noindent\colorbox{light-gray}{\parbox{17.5cm}{\textcolor{gray}{#1}}}}"#.to_string());

    vec![colors, my_red, line_color, light_gray, hll]
}

fn build_my_info(
    my_email: Option<String>,
    my_name: Option<String>,
    my_phone: Option<String>,
    my_country: Option<String>,
) -> Vec<PreambleElement> {
    let email = match my_email {
        Some(mail) => mail,
        None => "francesco@piva.online".to_string(),
    };

    let name = match my_name {
        Some(name) => name,
        None => "Francesco Piva".to_string(),
    };

    let phone = match my_phone {
        Some(number) => number,
        None => "(+41) 79 830 02 70".to_string(),
    };

    let country = match my_country {
        Some(place) => place,
        None => "Switzerland".to_string(),
    };

    vec![
        PreambleElement::UserDefined(
            format!(
                "\\makecvfooter{{{:}}}{{~~~·~~~\\textbf{{{:}}}~~~·~~~}}{{{:}}}",
                email, name, phone
            )
            .to_string(),
        ),
        PreambleElement::UserDefined(
            r#"\name{Francesco}{\\textcolor{awesome-red}{Piva}}"#.to_string(), // TODO: split name & surname
        ),
        PreambleElement::UserDefined(format!("\\address{{🌍 {:}}}", country).to_string()),
        PreambleElement::UserDefined(r#"\permis{Swiss, 37 years old}"#.to_string()),
        PreambleElement::UserDefined(format!("\\mobile{{{:}}}", phone).to_string()),
        PreambleElement::UserDefined(format!("\\email{{{:}}}", phone).to_string()),
    ]
}
