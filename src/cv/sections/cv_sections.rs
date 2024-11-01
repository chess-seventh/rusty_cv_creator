use latex::Element;
use crate::cv::helpers::ExperienceType;
use crate::cv::sections::{
    about_me::about_me, education::education, experiences::compile_experiences,
    extracurricular::extracurricular, skills::skills,
};

pub fn build_sections() -> Vec<Element> {
    let mut sections = vec![];

    let mut about_me = build_about_me();
    let mut skills = build_skills();
    let mut experience = build_experience(None);
    let mut education = build_education(None);
    let mut extra_curricular = build_extracurricular();

    sections.push(Element::UserDefined(
        r#"\makecvheader
        \vspace{-5mm}"#.to_string()));
    sections.append(&mut about_me);
    sections.push(Element::UserDefined(r#"\vspace{-3mm}"#.to_string()));
    sections.append(&mut skills);
    sections.append(&mut experience);
    sections.push(Element::UserDefined(r#"\vspace{-4mm}"#.to_string()));
    
    sections.append(&mut education);
    sections.push(Element::UserDefined(r#"\vspace{-4mm}"#.to_string()));
    
    sections.append(&mut extra_curricular);
    sections.push(Element::UserDefined(r#"\vspace{-5mm}"#.to_string()));

    sections
}

fn build_about_me() -> Vec<Element> {
    vec![about_me()]
}

fn build_skills() -> Vec<Element> {
    skills("main")
}

fn build_experience(experience_keyword: Option<&ExperienceType>) -> Vec<Element> {
    compile_experiences(experience_keyword)
}

fn build_education(education_selector: Option<&str>) -> Vec<Element> {
    education(education_selector)
}

fn build_extracurricular() -> Vec<Element> {
    vec![extracurricular()]
}
