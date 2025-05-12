use std::io::{self, Write};
use std::path::Path;
use serde::{Serialize,Deserialize};
use std::fs::File;
use std::time::SystemTime;
use std::collections::HashMap;
use std::fmt::Display;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq,PartialOrd, Ord)]
enum TaskStatus {
    Done,
    InProgress,
    New,
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",self)
        
        // match &self{
        //     TaskStatus::Done => write!(f, "{}", "Done"),
        //     TaskStatus::InProgress => write!(f, "{}", "In Progress"),
        //     TaskStatus::New => write!(f, "{}", "New"),
        // }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct TaskHash{
    hash: HashMap<i32,Task>,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    Quit,
}

enum ListOptions {
    Done,
    New,
    InProgress,
    All
}
#[derive(Debug,Error)]
enum ResponseError{
    #[error("Not a valid option")]
    InvalidInput,
    #[error("Task not found")]
    TaskNotFound,
    #[error("{0}")]
    ReadError(String),
    #[error("No Arguments Provided!")]
    NoArguments,
    #[error("{0}")]
    CreateFileError(String),
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Rust STD Library Error: {0}")]
    STDError(#[from] std::io::Error),
    #[error("Parsen Int Error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError)
}

fn process_response(res: &String) -> std::result::Result<MenuOptions, ResponseError>{
    let response_iter: Vec<_> = res.split(' ').collect();

    let menu_option = response_iter[0];

    let menu_option = match menu_option.trim().to_lowercase().as_str() {
        "add" => Some(MenuOptions::Add),
        "delete" => Some(MenuOptions::Delete),
        "update" => Some(MenuOptions::Update),
        "mark-done" => Some(MenuOptions::MarkDone),
        "mark-in-progress" => Some(MenuOptions::MarkInProgress),
        "list" => Some(MenuOptions::List),
        "quit" => Some(MenuOptions::Quit),
        "q" => Some(MenuOptions::Quit),
        _ => None,
    };

    if menu_option.is_none() {return Err(ResponseError::InvalidInput)}    

    Ok(menu_option.unwrap())

}

fn match_response(res:&str,menu_option: MenuOptions, path: &Path) -> std::result::Result<(), ResponseError> {
    match menu_option{
        MenuOptions::Add => add_task(&res,path),
        MenuOptions::Delete => delete_task(&res, path),
        MenuOptions::List => list_tasks(&res, path),
        MenuOptions::Quit => quit_process(),
        MenuOptions::Update => update_task_desc(&res, path),
        MenuOptions::MarkInProgress => mark_task_in_progress(&res,path),
        MenuOptions::MarkDone => mark_task_done(&res,path),
    }
}

fn mark_task_done(res: &str, path: &Path) -> std::result::Result<(), ResponseError>{
    let mut task_hash = file_contents_to_task_hash(&path)?;
    let response_vec: Vec<String> = true_input_parse(&res.to_string());
    let update_task_id: i32 = response_vec[1].clone().parse()?;
    let task = task_hash.hash.get(&update_task_id);
    
    if let None = task {
        return Err(ResponseError::TaskNotFound);
    }

    let task = task.unwrap();
    let updated_task = Task::new(task.id,task.description.clone(),TaskStatus::Done,task.created_at,SystemTime::now());
    let mut file = File::create(path)?;
    add_task_to_file(&mut file, &mut task_hash, &updated_task)?;
    println!("Task was updated to done!");
    Ok(())
}

fn mark_task_in_progress(res: &str, path: &Path) -> std::result::Result<(), ResponseError>{
    let mut task_hash = file_contents_to_task_hash(&path)?;
    let response_vec: Vec<String> = true_input_parse(&res.to_string());
    let update_task_id: i32 = response_vec[1].clone().parse()?;
    let task = task_hash.hash.get(&update_task_id);
    
    if let None = task {
        return Err(ResponseError::TaskNotFound);
    }

    let task = task.unwrap();
    let updated_task = Task::new(task.id,task.description.clone(),TaskStatus::InProgress,task.created_at,SystemTime::now());
    let mut file = File::create(path)?;
    add_task_to_file(&mut file, &mut task_hash, &updated_task)?;
    println!("Task was updated to in-progress!");
    Ok(())
}

fn update_task_desc(res: &str, path: &Path) -> std::result::Result<(), ResponseError> {
    let mut task_hash = file_contents_to_task_hash(&path)?;
    let response_vec: Vec<String> = true_input_parse(&res.to_string());
    let update_task_id: i32 = response_vec[1].clone().parse()?;
    let new_task_desc: String = response_vec[2].to_owned();
    let old_task = task_hash.hash.get(&update_task_id);
    if old_task.is_none(){
        return Err(ResponseError::TaskNotFound);
    }
    let old_task = old_task.unwrap();
    let new_task = Task::new(update_task_id, new_task_desc.clone(), old_task.status.clone(), old_task.created_at.clone(), SystemTime::now());
    let mut file = create_file(path)?;
    let old_task_desc = old_task.description.clone();   
    add_task_to_file(&mut file, &mut task_hash, &new_task)?;
    println!("Task was successfully updated from \"{}\" -> \"{}\"!", old_task_desc, new_task_desc);
    Ok(())
}

fn delete_task(res: &str, path: &Path) -> std::result::Result<(), ResponseError>{
    let mut task_hash = file_contents_to_task_hash(path)?;
    let response_vec:Vec<String> = true_input_parse(&res.to_string());
    let delete_task_id: i32 = response_vec[1].clone().parse()?;
    task_hash.hash.remove(&delete_task_id);
    let mut file = create_file(path)?;
    let json_task = serde_json::to_string(&task_hash)?;
    file.write_all(json_task.as_bytes())?;
    println!("Task was successfully deleted!");
    Ok(())
}

fn add_task(res: &str, path: &Path) -> std::result::Result<(),ResponseError> {
    //Create a Task Struct from input
    let str_vec = true_input_parse(&res.to_owned());
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
    let response_vector: Vec<String> = true_input_parse(&res.to_owned());
    if response_vector.len() == 1 {
        print_tasks(Some(ListOptions::All), task_hash)?;
        return Ok(())
    }
    let list_option = &response_vector[1];
    let list_option = match list_option.as_str(){
        "done" => Some(ListOptions::Done),
        "new" => Some(ListOptions::New),
        "in-progress" => Some(ListOptions::InProgress),
        _ => None,
    };
    if list_option.is_none() {return Err(ResponseError::InvalidInput)}
    print_tasks(list_option, task_hash)?;
    Ok(())
}

fn print_tasks(list_option: Option<ListOptions>, task_hash: TaskHash) -> std::result::Result<(), ResponseError>{
    if let Some(ListOptions::All) = list_option {
        let sorted_vector = task_hash_to_filtered_vector(&task_hash);
        for task in sorted_vector{
            println!("\nTask ID - {} \nDescription - {}\nStatus - {}\n",task.id, task.description, task.status);
        }
        return Ok(())
    } else if let Some(ListOptions::Done) = list_option {
        let done_tasks  = task_hash.hash.iter().filter(|(_,task)| task.status == TaskStatus::Done).collect();
        let done_tasks = filtered_vector_to_sorted_vec(done_tasks);
        println!("\n=== Completed Tasks ===");
        for task in done_tasks{
            println!("\nTask ID - {} \nDescription - {}\nStatus - {}\n",task.id, task.description, task.status);
        }
        return Ok(())
    } else if let Some(ListOptions::InProgress) = list_option {
        let in_progress_tasks  = task_hash.hash.iter().filter(|(_,task)| task.status == TaskStatus::InProgress).collect();
        let in_progress_tasks = filtered_vector_to_sorted_vec(in_progress_tasks);
        println!("\n=== Tasks In Progress ===");
        for task in in_progress_tasks{
            println!("\nTask ID - {} \nDescription - {}\nStatus - {}\n",task.id, task.description, task.status);
        }
        return Ok(())
    } else if let Some(ListOptions::New) = list_option {
        let new_tasks:Vec<(&i32, &Task)>  = task_hash.hash.iter().filter(|(_,task)| task.status == TaskStatus::New).collect();
        let new_tasks = filtered_vector_to_sorted_vec(new_tasks);
        println!("\n=== New Tasks ===");
        for task in new_tasks{
            println!("\nTask ID - {} \nDescription - {}\nStatus - {}\n",task.id, task.description, task.status);
        }
        return Ok(())
    } else {
        return Err(ResponseError::InvalidInput)
    };
}

fn filtered_vector_to_sorted_vec(filtered_vector: Vec<(&i32,&Task)>) -> Vec<Task> {
    let cloned_filtered_vector = filtered_vector.clone();
    let mut vector: Vec<Task> = vec![];
    for (_,task) in cloned_filtered_vector{
        vector.push(task.clone())
    }
    vector.sort();
    vector
}

fn task_hash_to_filtered_vector(task_hash: &TaskHash) -> Vec<Task> {
    let task_hash = task_hash.clone();
    let mut vector: Vec<Task> = vec![];
    for (_,task) in task_hash.hash.into_iter(){
        vector.push(task.clone())
    }
    vector.sort();
    vector
}

fn get_user_input() -> std::result::Result<String,ResponseError> {
    use std::io::stdin;
    let mut buffer = String::new();
    //Making sure that the carrot character is on the same line as the print statement
    print!("Task Tracker CLI - ");
    io::stdout().flush().unwrap();

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

//Splits the String into a vector while keeping non-empty substrings together as one item.
fn true_input_parse(string:&String) -> Vec<String> {
    // initialize variables to use
    let mut in_quotes = false;
    let mut current = String::new();
    let mut result: Vec<String> = vec![];
    // This variable allows us to see one iteration ahead
    let mut chars_iter = string.chars().peekable();

    //Loops through the peekable iteration
    while let Some(char) = chars_iter.next() {
        //If the next character exists and the current char is NOT a whitespace or a quotation mark
        if chars_iter.peek().is_none() && char != ' ' && char != '"'{
            //if the current word isn't empty after removing whitespaces and other "invisible" shit
            current.push(char);
            if !current.trim().is_empty(){
                //Push that shit into the result vector
                result.push(current.to_owned());
            }
            //Current word is cleared for the next word/substring
            current.clear();
        }
        //Puts us in quotation mode or takes us out of it
        if char == '"'{
            in_quotes = !in_quotes;
        } 
        //if the char is a whitespace or a quotation mark and if we aren't in quotation mod
        if (char == ' ' && in_quotes == false) || (char == '"' && in_quotes == false){
            if !current.is_empty(){
                result.push(current.clone());
            }
            current.clear();
            //If the character isn't a \ or a quote then push that shit to the current word
            } else if char != '\\' && char != '"' {
                current.push(char);
        }
    }
    //return result
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

fn quit_process() -> std::result::Result<(),ResponseError>{
    println!("CLI-Task Manager has been sucessfully closed");
    std::process::exit(0);
}
fn main() {
    let path = Path::new("./tasks.json");
    let inf_loop =true;
    while inf_loop {
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
        let result = match_response(&unwrapped_response,menu_option_response.unwrap(), &path);
        if let Err(e) = &result{
            println!("Error: {}", e);
            continue;
        }
    }
}
