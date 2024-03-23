use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, Write},
    path::Path,
};

use common::Account;

use super::{Crud, CrudError};

struct CsvAccount {
    line: u64,
    account: Account,
}

impl From<String> for CsvAccount {
    fn from(value: String) -> Self {
        CsvAccount {
            line: 0,
            account: Account::new(&value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CsvAccountStore {
    filename: String,
}

impl CsvAccountStore {
    pub fn new(filename: &str) -> Self {
        CsvAccountStore {
            filename: filename.to_string(),
        }
    }
}

impl Crud<Account> for CsvAccountStore {
    fn create(&mut self, account: &Account) -> super::Result<()> {
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
        if let Err(_) = writeln!(file, "{}", account.fullname) {
            Err(CrudError::UnknownError)
        } else {
            Ok(())
        }
    }

    fn read_all(&self) -> super::Result<Vec<Account>> {
        log::debug!("Reading Users from '{}'", &self.filename);
        let file = if Path::new(&self.filename).exists() {
            OpenOptions::new().read(true).open(&self.filename).unwrap()
        } else {
            return Ok(Vec::new());
        };

        let reader = BufReader::new(file);
        let mut accounts = Vec::new();

        for line in reader.lines() {
            if let Ok(fullname) = line {
                accounts.push(Account::new(&fullname));
            }
        }
        log::debug!("Read {} Users from '{}'", accounts.len(), &self.filename);
        Ok(accounts)
    }
    
    fn update(&mut self, item: &Account) -> super::Result<()> {
        let item = update_line(&self.filename, item)?;
        log::debug!("Updated line {} in {}", item.line, self.filename);
        Ok(())
    }

    fn delete(&mut self, item: &Account) -> super::Result<()> {
        delete_line(&self.filename, &item)
    }
}

// fn find_entry(path: &str, uuid: Uuid) -> super::Result<CsvAccount> {
//     let file = File::open(path)?;
//     let reader = BufReader::new(&file);
//     let mut line_count = 0;
//     for line in reader.lines() {
//         let line = line?;
//         if line.contains(&uuid.to_string()) {
//             return Ok(CsvAccount {
//                 line: line_count,
//                 account: Account::new(&line),
//             });
//         }
//         line_count += 1;
//     }
//     Err(CrudError::NotFound)
// }

fn update_line(path: &str, account: &Account) -> super::Result<CsvAccount> {
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
        if !line.contains(&account.id.to_string()) {
            writeln!(tempfile, "{}", line)?;
        } else {
            writeln!(tempfile, "{}", account.to_csv())?;
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
    Ok(CsvAccount {
        line: line_updated,
        account: account.to_owned(),
    })
}

fn delete_line(path: &str, account: &Account) -> super::Result<()> {
    // Open file in read mode
    let file = File::open(path)?;
    let reader = BufReader::new(&file);
    // Create temp file to write modified content
    let tempfile_path = format!("{}.tmp", path);
    let mut tempfile = File::create(&tempfile_path)?;
    // Iterate line by line, skipping the line that matches the uuid
    let mut line_count = 0;
    for line in reader.lines() {
        let line = line?;
        if !line.contains(&account.id.to_string()) {
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
    use uuid::uuid;

    use super::CsvAccountStore;

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
        let csv = dir.path().join("accounts.csv");
        let mut store = CsvAccountStore::new(csv.display().to_string().as_str());
        store
            .create(&Account::new("Test Account"))
            .expect("Failed to create new Account");
        assert!(csv.exists());
    }

    #[test]
    fn read_all_does_not_create_file_when_not_exist() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv = dir.path().join("accounts.csv");
        let store = CsvAccountStore::new(csv.display().to_string().as_str());
        store.read_all().expect("Failed to read Users");
        assert!(!csv.exists());
    }

    #[test]
    fn read_all_returns_none_when_file_not_exist() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv = dir.path().join("accounts.csv");
        let store = CsvAccountStore::new(csv.display().to_string().as_str());
        let accounts = store.read_all().expect("Failed to read Users");
        assert_eq!(accounts.len(), 0);
    }

    #[test]
    fn read_all_returns_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("accounts.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "Test Account").expect(&format!("Failed to write to {}", &csv_path.display()));
        let store = CsvAccountStore::new(csv_path.display().to_string().as_str());
        let accounts = store.read_all().expect("Failed to read Users");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts.get(0).unwrap().fullname, "Test Account");
    }

    #[test]
    fn update_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("accounts.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "67e55044-10b1-426f-9247-bb680e5fe0c8,Test Account").expect(&format!("Failed to write to {}", &csv_path.display()));
        let mut store = CsvAccountStore::new(csv_path.display().to_string().as_str());
        let mut user_updated = Account::new("Modified Account");
        user_updated.id = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.update(&user_updated).expect("Failed to update Account");
        let actual_line = read_line(csv_path.display().to_string().as_str(), 0).expect(&format!("Failed to read line 0 from {}", &csv_path.display()));
        assert_eq!(actual_line, "67e55044-10b1-426f-9247-bb680e5fe0c8,Modified Account");
    }

    #[test]
    fn update_one_of_two() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("accounts.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "67e55044-10b1-426f-9247-bb680e5fe0c8,Test Account 1\n67e55044-10b1-426f-9247-bb680e5fe0c9,Test Account 2").expect(&format!("Failed to write to {}", &csv_path.display()));
        let mut store = CsvAccountStore::new(csv_path.display().to_string().as_str());
        let mut user_updated = Account::new("Modified Account 1");
        user_updated.id = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.update(&user_updated).expect("Failed to update Account");
        let actual_line = read_line(csv_path.display().to_string().as_str(), 0).expect(&format!("Failed to read line 0 from {}", &csv_path.display()));
        assert_eq!(actual_line, "67e55044-10b1-426f-9247-bb680e5fe0c8,Modified Account 1");
    }

    #[test]
    fn delete_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("accounts.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "67e55044-10b1-426f-9247-bb680e5fe0c8,Test Account").expect(&format!("Failed to write to {}", &csv_path.display()));
        let mut store = CsvAccountStore::new(csv_path.display().to_string().as_str());
        let mut account = Account::new("Test Account");
        account.id = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.delete(&account).expect("Failed to delete Account");
        assert_eq!(
            count_lines(csv_path.display().to_string().as_str())
                .expect(&format!("Failed to count lines of {}", &csv_path.display())),
            0
        );
    }

    #[test]
    fn delete_one_of_two() {
        let dir = tempdir().expect("Failed to create temp directory");
        let csv_path = dir.path().join("accounts.csv");
        let mut csv =
            File::create(&csv_path).expect(&format!("Failed to create {}", &csv_path.display()));
        writeln!(csv, "67e55044-10b1-426f-9247-bb680e5fe0c8,Test Account 1\n67e55044-10b1-426f-9247-bb680e5fe0c9,Test Account 2")
            .expect(&format!("Failed to write to {}", &csv_path.display()));
        let mut store = CsvAccountStore::new(csv_path.display().to_string().as_str());
        let mut account = Account::new("Test Account 1");
        account.id = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.delete(&account).expect("Failed to delete Account");
        assert_eq!(
            count_lines(csv_path.display().to_string().as_str())
                .expect(&format!("Failed to count lines of {}", &csv_path.display())),
            1
        );
    }
}
