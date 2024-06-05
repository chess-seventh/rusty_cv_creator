use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct UserInput {
    #[arg(short, long)]
    #[arg(default_value_t = false)]
    pub save_to_database: bool,

    #[arg(short, long)]
    #[arg(default_value_t = String::from("./dummy_config.ini"))]
    pub config_ini: String,

    #[command(subcommand)]
    pub action: UserAction,

}

#[derive(Subcommand, Debug)]
pub enum UserAction {
    Insert(InsertCV),
    Update(UpdateCV),
    Remove(RemoveCV),
    // List(ListCV),
}

#[derive(Args, Debug)]
pub struct InsertCV {
    pub company_name: String,
    pub job_title: String,
    pub quote: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateCV {
    pub job_title: String,
    pub company_name: String,
    pub quote: Option<String>,
}

#[derive(Args, Debug)]
pub struct RemoveCV {
    pub job_title: String,
    pub company_name: String,
}

pub fn match_user_action(action: UserAction) -> (String, String, Option<String>) {
    match action {
            UserAction::Insert(insert) => {
                match_insert(insert)
            }
            UserAction::Update(_update) => {
                todo!();
            }
            UserAction::Remove(_remove) => {
                todo!();
            }
    }
}

fn match_insert(insert: InsertCV) -> (String, String, Option<String>){
    (insert.job_title, insert.company_name, insert.quote)
}

fn _match_update(update: &UpdateCV) {
    println!("Updating CV: {update:#?}");
}

fn _match_remove(remove: &RemoveCV) {
    println!("Removing CV: {remove:#?}");
}
