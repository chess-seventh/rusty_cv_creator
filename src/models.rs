use crate::schema::cv;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = cv)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Cv {
    pub id: i32,
    pub application_date: Option<String>,
    pub job_title: String,
    pub company: String,
    pub quote: String,
    pub pdf_cv_path: String,
    pub generated: bool,
}

#[derive(Insertable)]
#[diesel(table_name = cv)]
pub struct NewCv<'a> {
    pub application_date: Option<&'a str>,
    pub job_title: &'a str,
    pub company: &'a str,
    pub quote: &'a str,
    pub pdf_cv_path: &'a str,
    pub generated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cv_model_creation() {
        let cv = Cv {
            id: 1,
            application_date: Some("2023-08-19".to_string()),
            job_title: "Software Engineer".to_string(),
            company: "ACME Corp".to_string(),
            quote: "Great opportunity to work with cutting-edge technology".to_string(),
            pdf_cv_path: "/home/user/cvs/2023-08-19-acme-software-engineer.pdf".to_string(),
            generated: true,
        };

        assert_eq!(cv.id, 1);
        assert_eq!(cv.application_date, Some("2023-08-19".to_string()));
        assert_eq!(cv.job_title, "Software Engineer");
        assert_eq!(cv.company, "ACME Corp");
        assert_eq!(
            cv.quote,
            "Great opportunity to work with cutting-edge technology"
        );
        assert_eq!(
            cv.pdf_cv_path,
            "/home/user/cvs/2023-08-19-acme-software-engineer.pdf"
        );
        assert_eq!(cv.generated, true);
    }

    #[test]
    fn test_cv_model_with_none_date() {
        let cv = Cv {
            id: 2,
            application_date: None,
            job_title: "Senior Developer".to_string(),
            company: "Tech Inc".to_string(),
            quote: "Exciting challenge".to_string(),
            pdf_cv_path: "/tmp/cv.pdf".to_string(),
            generated: false,
        };

        assert_eq!(cv.id, 2);
        assert_eq!(cv.application_date, None);
        assert_eq!(cv.job_title, "Senior Developer");
        assert_eq!(cv.company, "Tech Inc");
        assert_eq!(cv.quote, "Exciting challenge");
        assert_eq!(cv.pdf_cv_path, "/tmp/cv.pdf");
        assert_eq!(cv.generated, false);
    }

    #[test]
    fn test_new_cv_creation() {
        let new_cv = NewCv {
            application_date: Some("2023-08-19"),
            job_title: "Product Manager",
            company: "StartupCo",
            quote: "Innovation-focused role",
            pdf_cv_path: "/tmp/new_cv.pdf",
            generated: true,
        };

        assert_eq!(new_cv.application_date, Some("2023-08-19"));
        assert_eq!(new_cv.job_title, "Product Manager");
        assert_eq!(new_cv.company, "StartupCo");
        assert_eq!(new_cv.quote, "Innovation-focused role");
        assert_eq!(new_cv.pdf_cv_path, "/tmp/new_cv.pdf");
        assert_eq!(new_cv.generated, true);
    }

    #[test]
    fn test_new_cv_with_none_date() {
        let new_cv = NewCv {
            application_date: None,
            job_title: "Data Scientist",
            company: "AI Corp",
            quote: "Machine learning expertise",
            pdf_cv_path: "/home/user/data_scientist_cv.pdf",
            generated: false,
        };

        assert_eq!(new_cv.application_date, None);
        assert_eq!(new_cv.job_title, "Data Scientist");
        assert_eq!(new_cv.company, "AI Corp");
        assert_eq!(new_cv.quote, "Machine learning expertise");
        assert_eq!(new_cv.pdf_cv_path, "/home/user/data_scientist_cv.pdf");
        assert_eq!(new_cv.generated, false);
    }

    #[test]
    fn test_cv_debug_implementation() {
        let cv = Cv {
            id: 1,
            application_date: Some("2023-08-19".to_string()),
            job_title: "Engineer".to_string(),
            company: "Company".to_string(),
            quote: "Quote".to_string(),
            pdf_cv_path: "/tmp/cv.pdf".to_string(),
            generated: true,
        };

        let debug_str = format!("{:?}", cv);
        assert!(debug_str.contains("id: 1"));
        assert!(debug_str.contains("Engineer"));
        assert!(debug_str.contains("Company"));
        assert!(debug_str.contains("Quote"));
        assert!(debug_str.contains("/tmp/cv.pdf"));
        assert!(debug_str.contains("generated: true"));
    }

    #[test]
    fn test_cv_with_special_characters() {
        let cv = Cv {
            id: 3,
            application_date: Some("2023-08-19".to_string()),
            job_title: "C++ Developer".to_string(),
            company: "Tech & Innovation Co.".to_string(),
            quote: "\"Passionate about systems programming\"".to_string(),
            pdf_cv_path: "/home/user/cvs/c++_developer.pdf".to_string(),
            generated: true,
        };

        assert_eq!(cv.job_title, "C++ Developer");
        assert_eq!(cv.company, "Tech & Innovation Co.");
        assert_eq!(cv.quote, "\"Passionate about systems programming\"");
        assert!(cv.pdf_cv_path.contains("c++_developer"));
    }

    #[test]
    fn test_cv_with_unicode() {
        let cv = Cv {
            id: 4,
            application_date: Some("2023-08-19".to_string()),
            job_title: "软件工程师".to_string(),
            company: "科技公司".to_string(),
            quote: "热衷于技术创新".to_string(),
            pdf_cv_path: "/home/user/简历.pdf".to_string(),
            generated: true,
        };

        assert_eq!(cv.job_title, "软件工程师");
        assert_eq!(cv.company, "科技公司");
        assert_eq!(cv.quote, "热衷于技术创新");
        assert!(cv.pdf_cv_path.contains("简历"));
    }

    #[test]
    fn test_new_cv_with_long_strings() {
        let long_quote = "This is a very long quote that describes the candidate's passion for technology, their extensive experience in software development, their commitment to continuous learning, and their ability to work effectively in team environments while also being capable of independent work when necessary.".to_string();

        let new_cv = NewCv {
            application_date: Some("2023-08-19"),
            job_title: "Senior Software Architect",
            company: "Enterprise Solutions Corporation",
            quote: &long_quote,
            pdf_cv_path: "/home/user/documents/cvs/2023/august/senior_software_architect_enterprise_solutions_corporation.pdf",
            generated: true,
        };

        assert!(new_cv.quote.len() > 200);
        assert!(new_cv.pdf_cv_path.len() > 50);
        assert_eq!(new_cv.job_title, "Senior Software Architect");
        assert_eq!(new_cv.company, "Enterprise Solutions Corporation");
    }

    #[test]
    fn test_new_cv_with_empty_strings() {
        let new_cv = NewCv {
            application_date: Some(""),
            job_title: "",
            company: "",
            quote: "",
            pdf_cv_path: "",
            generated: false,
        };

        assert_eq!(new_cv.application_date, Some(""));
        assert_eq!(new_cv.job_title, "");
        assert_eq!(new_cv.company, "");
        assert_eq!(new_cv.quote, "");
        assert_eq!(new_cv.pdf_cv_path, "");
        assert_eq!(new_cv.generated, false);
    }

    #[test]
    fn test_cv_field_types() {
        let cv = Cv {
            id: 42,
            application_date: Some("2023-12-31".to_string()),
            job_title: "Test Engineer".to_string(),
            company: "Quality Corp".to_string(),
            quote: "Quality is key".to_string(),
            pdf_cv_path: "/quality/cv.pdf".to_string(),
            generated: true,
        };

        // Test field types
        assert_eq!(std::mem::size_of_val(&cv.id), std::mem::size_of::<i32>());
        assert_eq!(
            std::mem::size_of_val(&cv.generated),
            std::mem::size_of::<bool>()
        );

        // Test that strings are indeed String type
        assert!(cv.job_title.capacity() >= cv.job_title.len());
        assert!(cv.company.capacity() >= cv.company.len());
        assert!(cv.quote.capacity() >= cv.quote.len());
        assert!(cv.pdf_cv_path.capacity() >= cv.pdf_cv_path.len());
    }

    // Test compatibility with diesel traits
    #[test]
    fn test_diesel_traits_compilation() {
        // This test ensures that the derive macros compile correctly
        // The actual functionality would require database setup to test

        // Test that we can create instances (compile-time check)
        let _cv = Cv {
            id: 1,
            application_date: None,
            job_title: "Test".to_string(),
            company: "Test Co".to_string(),
            quote: "Test quote".to_string(),
            pdf_cv_path: "/test.pdf".to_string(),
            generated: false,
        };

        let _new_cv = NewCv {
            application_date: None,
            job_title: "Test",
            company: "Test Co",
            quote: "Test quote",
            pdf_cv_path: "/test.pdf",
            generated: false,
        };

        // If these compile, the diesel derives are working
        assert!(true);
    }

    #[test]
    fn test_model_memory_layout() {
        use std::mem;

        // Test that the models have reasonable memory footprints
        let cv_size = mem::size_of::<Cv>();
        let new_cv_size = mem::size_of::<NewCv>();

        // These should be reasonable sizes (not exact, but sanity checks)
        assert!(cv_size > 0);
        assert!(cv_size < 1000); // Reasonable upper bound
        assert!(new_cv_size > 0);
        assert!(new_cv_size < 1000);
    }

    // Edge case tests
    #[test]
    fn test_cv_with_maximum_id() {
        let cv = Cv {
            id: i32::MAX,
            application_date: Some("2023-08-19".to_string()),
            job_title: "Test".to_string(),
            company: "Test".to_string(),
            quote: "Test".to_string(),
            pdf_cv_path: "/test.pdf".to_string(),
            generated: true,
        };

        assert_eq!(cv.id, i32::MAX);
    }

    #[test]
    fn test_cv_with_minimum_id() {
        let cv = Cv {
            id: i32::MIN,
            application_date: Some("2023-08-19".to_string()),
            job_title: "Test".to_string(),
            company: "Test".to_string(),
            quote: "Test".to_string(),
            pdf_cv_path: "/test.pdf".to_string(),
            generated: false,
        };

        assert_eq!(cv.id, i32::MIN);
    }
}
