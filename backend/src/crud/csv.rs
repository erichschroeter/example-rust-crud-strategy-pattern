use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, Write},
    path::Path,
};

use common::User;
use uuid::Uuid;

use super::{Crud, CrudError};

struct CsvUser {
    line: u64,
    user: User,
}

impl From<String> for CsvUser {
    fn from(value: String) -> Self {
        CsvUser {
            line: 0,
            user: User::new(&value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CsvUserStore {
    filename: String,
}

impl CsvUserStore {
    pub fn new(filename: &str) -> Self {
        CsvUserStore {
            filename: filename.to_string(),
        }
    }
}

impl Crud<User> for CsvUserStore {
    fn create(&mut self, user: &User) -> super::Result<()> {
        let mut file = if Path::new(&self.filename).exists() {
            OpenOptions::new()
                .write(true)
                .append(true)
                .open(&self.filename)
                .unwrap()
        } else {
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&self.filename)
                .unwrap()
        };
        if let Err(_) = writeln!(file, "{}", user.fullname) {
            Err(CrudError::UnknownError)
        } else {
            Ok(())
        }
    }

    fn read_all(&self) -> super::Result<Vec<User>> {
        log::debug!("Reading Users from '{}'", &self.filename);
        let file = if Path::new(&self.filename).exists() {
            OpenOptions::new().read(true).open(&self.filename).unwrap()
        } else {
            return Ok(Vec::new());
        };

        let reader = BufReader::new(file);
        let mut users = Vec::new();

        for line in reader.lines() {
            if let Ok(fullname) = line {
                users.push(User::new(&fullname));
            }
        }
        log::debug!("Read {} Users from '{}'", users.len(), &self.filename);
        Ok(users)
    }
    
    fn update(&mut self, item: &User) -> super::Result<()> {
        let item = update_line(&self.filename, item)?;
        Ok(())
    }

    fn delete(&mut self, item: &User) -> super::Result<()> {
        delete_line(&self.filename, &item.fullname)
    }
}

// fn find_entry(path: &str, uuid: Uuid) -> super::Result<CsvUser> {
//     let file = File::open(path)?;
//     let reader = BufReader::new(&file);
//     let mut line_count = 0;
//     for line in reader.lines() {
//         let line = line?;
//         if line.contains(&uuid.to_string()) {
//             return Ok(CsvUser {
//                 line: line_count,
//                 user: User::new(&line),
//             });
//         }
//         line_count += 1;
//     }
//     Err(CrudError::NotFound)
// }

fn update_line(path: &str, user: &User) -> super::Result<CsvUser> {
    // Open file in read mode
    let file = File::open(path)?;
    let reader = BufReader::new(&file);
    // Create temp file to write modified content
    let tempfile_path = format!("{}.tmp", path);
    let mut tempfile = File::create(&tempfile_path)?;
    // Iterate line by line, skipping the line that matches the username
    let mut line_count = 0;
    let mut line_updated = 0;
    for line in reader.lines() {
        let line = line?;
        if !line.contains(&user.id.to_string()) {
            writeln!(tempfile, "{}", line)?;
        } else {
            writeln!(tempfile, "{}", user.to_csv())?;
            line_updated = line_count;
            log::debug!("Updated line {}", line_count)
        }
        line_count += 1;
    }
    // Truncate the original file and copy the modified content from temp file
    let mut file = File::create(path)?;
    file.seek(std::io::SeekFrom::Start(0))?;
    std::io::copy(&mut File::open(&tempfile_path)?, &mut file)?;
    std::fs::remove_file(&tempfile_path)?;
    Ok(CsvUser {
        line: line_updated,
        user: user.to_owned(),
    })
}

fn delete_line(path: &str, username: &str) -> super::Result<()> {
    // Open file in read mode
    let file = File::open(path)?;
    let reader = BufReader::new(&file);
    // Create temp file to write modified content
    let tempfile_path = format!("{}.tmp", path);
    let mut tempfile = File::create(&tempfile_path)?;
    // Iterate line by line, skipping the line that matches the username
    let mut line_count = 0;
    for line in reader.lines() {
        let line = line?;
        if !line.contains(username) {
            writeln!(tempfile, "{}", line)?;
        } else {
            log::debug!("Deleted line {}", line_count)
        }
        line_count += 1;
    }
    // Truncate the original file and copy the modified content from temp file
    let mut file = File::create(path)?;
    file.seek(std::io::SeekFrom::Start(0))?;
    std::io::copy(&mut File::open(&tempfile_path)?, &mut file)?;
    std::fs::remove_file(&tempfile_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempfile::tempdir;

    use super::CsvUserStore;

    use super::*;

    fn count_lines(path: &str) -> std::io::Result<usize> {
        let mut count = 0;
        for line in std::io::BufReader::new(std::fs::File::open(&path)?).lines() {
            line?;
            count += 1;
        }
        Ok(count)
    }

    fn read_line(path: &str, line_no: usize) ->std::io::Result<String> {
        let mut count = 0;
        for line in std::io::BufReader::new(std::fs::File::open(&path)?).lines() {
            let line = line?;
            if count == line_no {
                return Ok(line)
            }
            count += 1;
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File only {} lines", count)))
    }

    #[test]
    fn create_creates_file_when_not_exist() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv = dir.path().join("users.csv");
        let mut store = CsvUserStore::new(csv.display().to_string().as_str());
        store
            .create(&User::new("Test User"))
            .expect("Failed to create new User");
        assert!(csv.exists());
    }

    #[test]
    fn read_all_does_not_create_file_when_not_exist() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv = dir.path().join("users.csv");
        let store = CsvUserStore::new(csv.display().to_string().as_str());
        store.read_all().expect("Failed to read Users");
        assert!(!csv.exists());
    }

    #[test]
    fn read_all_returns_none_when_file_not_exist() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv = dir.path().join("users.csv");
        let store = CsvUserStore::new(csv.display().to_string().as_str());
        let users = store.read_all().expect("Failed to read Users");
        assert_eq!(users.len(), 0);
    }

    #[test]
    fn read_all_returns_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("users.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "Test User").expect(&format!("Failed to write to {}", &csv_path.display()));
        let store = CsvUserStore::new(csv_path.display().to_string().as_str());
        let users = store.read_all().expect("Failed to read Users");
        assert_eq!(users.len(), 1);
        assert_eq!(users.get(0).unwrap().fullname, "Test User");
    }

    #[test]
    fn update_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("users.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "67e55044-10b1-426f-9247-bb680e5fe0c8,Test User").expect(&format!("Failed to write to {}", &csv_path.display()));
        let mut store = CsvUserStore::new(csv_path.display().to_string().as_str());
        let mut user_updated = User::new("Modified User");
        user_updated.id = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.update(&user_updated).expect("Failed to update User");
        let actual_line = read_line(csv_path.display().to_string().as_str(), 0).expect(&format!("Failed to read line 0 from {}", &csv_path.display()));
        assert_eq!(actual_line, "67e55044-10b1-426f-9247-bb680e5fe0c8,Modified User");
    }

    #[test]
    fn delete_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("users.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "Test User").expect(&format!("Failed to write to {}", &csv_path.display()));
        let mut store = CsvUserStore::new(csv_path.display().to_string().as_str());
        let user = User::new("Test User");
        store.delete(&user).expect("Failed to delete User");
        assert_eq!(
            count_lines(csv_path.display().to_string().as_str())
                .expect(&format!("Failed to count lines of {}", &csv_path.display())),
            0
        );
    }

    #[test]
    fn delete_one_of_two() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("users.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "Test User 1\nTest User 2")
            .expect(&format!("Failed to write to {}", &csv_path.display()));
        let mut store = CsvUserStore::new(csv_path.display().to_string().as_str());
        let user = User::new("Test User 1");
        store.delete(&user).expect("Failed to delete User");
        assert_eq!(
            count_lines(csv_path.display().to_string().as_str())
                .expect(&format!("Failed to count lines of {}", &csv_path.display())),
            1
        );
    }
}
