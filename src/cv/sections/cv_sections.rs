use latex::Element;

use crate::cv::sections::{
    about_me::about_me, education::education, experiences::experience,
    extracurricular::extracurricular, skills::skills,
};

pub fn build_sections() -> Vec<Element> {
    let mut sections = vec![];

    let mut about_me = build_about_me();
    let mut skills = build_skills();
    let mut experience = build_experience();
    let mut education = build_education();
    let mut extra_curricular = build_extracurricular();

    sections.append(&mut about_me);
    sections.append(&mut skills);
    sections.append(&mut experience);
    sections.append(&mut education);
    sections.append(&mut extra_curricular);

    sections
}

fn build_about_me() -> Vec<Element> {
    vec![about_me()]
}

fn build_skills() -> Vec<Element> {
    vec![skills()]
}

fn build_experience() -> Vec<Element> {
    vec![experience()]
}

fn build_education() -> Vec<Element> {
    vec![education()]
}

fn build_extracurricular() -> Vec<Element> {
    vec![extracurricular()]
}
