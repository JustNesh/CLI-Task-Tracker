use std::fmt::write;
use std::hash::Hash;
use std::io::{Error, Write};
use std::ops::Index;
use std::path::Path;
use std::task;
use serde::{Serialize,Deserialize};
use std::fs::{File,OpenOptions};
use serde_json::Result;
use std::time::SystemTime;
use std::collections::HashMap;

use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TaskStatus {
    Done,
    InProgress,
    New,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TaskHash{
    hash: HashMap<i32,Task>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task{
    id: i32,
    description: String,
    status: TaskStatus,
    created_at: SystemTime,
    updated_at: SystemTime,
}

impl Task {
    fn new(            
        id:i32, description:String, status:TaskStatus, 
        created_at: SystemTime, updated_at:SystemTime,) -> Self {
        Self {id,description,status,created_at,updated_at}
    }
}
#[derive(Debug)]
enum MenuOptions{
    Add,
    Delete,
    Update,
    MarkDone,
    MarkInProgress,
    List,
}

enum ListOptions {
    Done,
    Todo,
    InProgress,
    All
}
#[derive(Debug,Error)]
enum ResponseError{
    #[error("Not a valid option")]
    InvalidInput,
    #[error("{0}")]
    ReadError(String),
    #[error("No Arguments Provided!")]
    NoArguments,
    #[error("{0}")]
    CreateFileError(String),
    #[error("{0}")]
    JsonParsingError(String),
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Rust STD Library Error: {0}")]
    STDError(#[from] std::io::Error),
}

fn process_response(res: &String) -> std::result::Result<MenuOptions, ResponseError>{
    let response_iter: Vec<_> = res.split(' ').collect();

    let menu_option = response_iter[0];

    let menu_option = match menu_option.trim().to_lowercase().as_str() {
        "add" => Some(MenuOptions::Add),
        "delete" => Some(MenuOptions::Delete),
        "update" => Some(MenuOptions::Update),
        "markdone" => Some(MenuOptions::MarkDone),
        "markinprogress" => Some(MenuOptions::MarkInProgress),
        "list" => Some(MenuOptions::List),
        _ => None,
    };

    if menu_option.is_none() {return Err(ResponseError::InvalidInput)}    

    Ok(menu_option.unwrap())

}

// fn match_response(res:&str,menu_option: MenuOptions, path: &Path) {
//     match menu_option{
//         MenuOptions::Add => add_task(res,path),
//         MenuOptions::Delete => delete_task(),
//         MenuOptions::List => list_tasks(),
//         MenuOptions::Update => update_task(),
//         MenuOptions::MarkInProgress => change_task_status(),
//         MenuOptions::MarkDone => change_task_status(),
//     }
// }

fn match_response(res:&str,menu_option: MenuOptions, path: &Path) -> std::result::Result<(), ResponseError> {
    match menu_option{
        MenuOptions::Add => add_task(&res,path),
        MenuOptions::List => list_tasks(&res, path),
        _ => add_task(res, path),
    }
}

fn add_task(res: &str, path: &Path) -> std::result::Result<(),ResponseError> {
    //Create a Task Struct from input
    let str_vec = parse_user_input(&res.to_owned());
    let task_string = &str_vec[1];    
    if !path.exists(){
        let new_task = Task::new(1, task_string.to_owned(), TaskStatus::New, SystemTime::now(), SystemTime::now());
        //Create a new file
        let mut file = create_file(path)?;
        let mut task_hash = TaskHash {hash: HashMap::new()};
        //Add Task to File
        add_task_to_file(&mut file, &mut task_hash, &new_task)?;
        println!("Task was added successfully! Task ID: {} - Task Name: {}", &new_task.id, &new_task.description);
        Ok(())
    } else {
        //Reading the Tasks
        let mut task_hash: TaskHash = file_contents_to_task_hash(path)?;
        //Iterating over tasks to create the new ID number
        let new_id:i32 = create_new_id(&task_hash);
        //Creates a New Task
        let new_task = Task::new(new_id, task_string.to_owned(), TaskStatus::New, SystemTime::now(), SystemTime::now());
        //Rewrite file from scatch
        let mut file = create_file(path)?;
        //Add Task to File
        task_hash.hash.insert(new_task.id.clone(), new_task.clone());
        add_task_to_file(&mut file, &mut task_hash, &new_task)?;
        println!("Task was added successfully! Task ID: {} - Task Name: {}", &new_task.id, &new_task.description);
        Ok(())
    }
}


fn list_tasks(res: &str, path: &Path) -> std::result::Result<(),ResponseError>{
    //Reading the Tasks
    let task_hash = file_contents_to_task_hash(path)?;
    //Parsing the User's input and checking to see if the user 
    //wants to see all the tasks or something else
    let response_vector: Vec<String> = parse_user_input(&res.to_owned());
    if response_vector.len() == 1 {
        for (_,task) in task_hash.hash{
            println!("\nTask ID - {} \nDescription - {}\n",task.id, task.description);
        }    
        return Ok(())
    }
    let list_option = &response_vector[1];
    let list_option = match list_option.as_str(){
        "done" => Some(ListOptions::Done),
        "todo" => Some(ListOptions::Todo),
        "inprogress" => Some(ListOptions::InProgress),
        _ => None,
    };

    if list_option.is_none() {return Err(ResponseError::InvalidInput)}
    Ok(())
}

///JUSTIN USE THIS FUNCTION TO USE THE RUST SORT BY VEC METHOD TO SORT THE VECTOR AND THEN PRINT OUT IN ORDER BY ID
fn task_hash_to_sorted_vec(task_hash: &TaskHash) -> Vec<Task>{
    let vec:Vec<Task> = vec![];
    let task_hash_len:i32 = task_hash.hash.len().clone() as i32;
    for num in 0..=task_hash_len{
        let result = &task_hash.hash.get_key_value(&num);
        if let None = result {
            continue
        } else {
            let result =result.unwrap();
        }
    }
}


fn get_user_input() -> std::result::Result<String,ResponseError> {
    use std::io::stdin;
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer){
        Ok(_) => {
            if buffer.trim().len() == 0{
                return Err(ResponseError::NoArguments)
            }
            Ok(buffer.trim().to_owned())
        }
        Err(e) => return Err(ResponseError::ReadError(format!("{}",e))),
    }
}

fn parse_user_input(ui:&String) -> Vec<String>{
    let mut is_parenthesis = false;
    let mut is_beginning_of_word = false;
    let mut beginning_of_word: i16 = 0_i16;
    let mut end_of_word: i16 = 0_i16;
    let mut vector_of_words: Vec<String> = vec![]; 
    let mut beginning_of_string: bool = true;
    let mut parenthesis_count = 0;
    for char in ui.chars() {
        if char == '"' {
            is_parenthesis = !is_parenthesis;
            if parenthesis_count == 1 {

            }
            parenthesis_count +=1;
            continue
        }
        if is_parenthesis{
            continue
        }
        if char == ' ' && is_beginning_of_word{
            beginning_of_word = ui.find(char).unwrap() as i16 + 1_i16;
        } else if char == ' ' && !is_beginning_of_word{
            end_of_word = ui.find(char).unwrap() as i16;
            vector_of_words.push(ui[beginning_of_word as usize..end_of_word as usize].to_owned()).to_owned();
        }

    }
    let vec: Vec<_> = ui.split(' ').map(|item| item.to_owned()).collect();
    vec
}

fn test_parse(string: &String) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut in_quotes = false;

    for c in string.chars() {
        if c == '"' {
            in_quotes = !in_quotes; // Toggle quoted state
        } else if c == ' ' && !in_quotes {
            if !current_word.is_empty() {
                result.push(current_word.clone());
                current_word.clear();
            }
        } else {
            current_word.push(c);
        }
    }
    result
}

fn create_file(path:&Path) -> std::result::Result<File,ResponseError>{
        //Create a file
        let file = File::create(path);
        if let Err(e) = file {
            return Err(ResponseError::CreateFileError(e.to_string()))
        }
        let file = file.unwrap();
        Ok(file)
}

fn add_task_to_file(file:&mut File, task_hash:&mut TaskHash, new_task:&Task) -> std::result::Result<(),ResponseError>{
    task_hash.hash.insert(new_task.id.clone(), new_task.clone());
    let json_task = serde_json::to_string(&task_hash)?;
    file.write_all(json_task.as_bytes())?;
    Ok(())
}

///Takes the file path and converts file to TaskHash 
fn file_contents_to_task_hash(path:&Path) -> std::result::Result<TaskHash, ResponseError>{
    let file_contents_to_string = std::fs::read_to_string(path)?;
    let task_hash: TaskHash = serde_json::from_str(file_contents_to_string.as_str())?;
    Ok(task_hash)
}

///Creates a new ID based off of all task IDs from the TaskHash
fn create_new_id(task_hash:&TaskHash) -> i32 {
    let mut highest_id = 0 as i32;
    for (id,_) in &task_hash.hash{
        if *id >= highest_id {
            highest_id = *id
        }
    }
    let new_id:i32 = highest_id + 1;
    new_id
}
fn main() {

    // let result = test_parse(&"\" THIS IS A STRINGGG\" 1 2 3".to_owned());
    // println!("{:?}", result);

    let path = Path::new("./tasks.json");
    while true{
        let response = get_user_input();
        if let Err(e) = &response {
            println!("Error: {}", e);
            continue;
        } 
        let unwrapped_response = &response.unwrap();
        let menu_option_response = process_response(unwrapped_response);
        if let Err(e) = &menu_option_response {
            println!("Error: {}", e);
            continue;
        }
        match_response(&unwrapped_response,menu_option_response.unwrap(), &path);
    }
}
