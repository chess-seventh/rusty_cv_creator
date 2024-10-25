use latex::Element;

use crate::cv::helpers::{experience_header, ExperienceType};

use crate::cv::sections::experiences_2017_2017_antony::antony;
use crate::cv::sections::experiences_2017_2019_ebu::ebu;
use crate::cv::sections::experiences_2019_2019_pictet::pictet;
use crate::cv::sections::experiences_2019_2021_bestmile::bestmile;
use crate::cv::sections::experiences_2021_2022_open_systems::open_systems;
use crate::cv::sections::experiences_2022_2024_zf::zf;
use crate::cv::sections::experiences_2024_piva_consulting::piva_consulting;
use crate::cv::sections::experiences_2007_hospitality::experience_hospitality;

use crate::cv::helpers::experience_footer;

pub fn compile_experiences(experience_keyword: Option<&ExperienceType>) -> Vec<Element> {
    let mut experiences = vec![];

    experiences.push(experience_header());

    experiences.append(&mut build_experiences(experience_keyword));

    experiences.push(experience_footer());

    experiences
}

pub fn build_experiences(experience_keyword: Option<&ExperienceType>) -> Vec<Element> {
    let mut building_exp = vec![];

    building_exp.append(&mut piva_consulting(experience_keyword));
    building_exp.append(&mut zf(experience_keyword));
    building_exp.append(&mut open_systems(experience_keyword));
    building_exp.append(&mut bestmile(experience_keyword));
    building_exp.append(&mut pictet(experience_keyword));
    building_exp.append(&mut ebu(experience_keyword));
    building_exp.append(&mut antony(experience_keyword));

    if let Some(ExperienceType::FullHospitality) = experience_keyword {
        building_exp.append(&mut experience_hospitality());
    }

    building_exp
}
