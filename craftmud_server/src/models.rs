use std::error::Error;

use crossbeam_channel::{self as channel, Receiver, TryRecvError,};
use tokio_postgres::error::{DbError, Error as TpgError};

use crate::login;
use crate::outside::Database;

type Response<T> = Receiver<Result<T, tokio_postgres::Error>>;

pub struct Account {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug)]
pub struct AccountInsert(Response<u64>);

impl AccountInsert {
    pub fn try_recv(&self) -> Result<Result<(), UniqueAccountError>, TryRecvError> {
        self.0.try_recv().map(|res| {
            match res {
                Ok(_) => Ok(()),
                Err(postgres_err) => {
                    match postgres_err.source()
                    .and_then(|e| e.downcast_ref::<DbError>())
                    .and_then(|e| e.constraint())
                    {
                        None => {panic!("CheckUnique query failed."); },
                        Some("accounts_pkey") => Err(UniqueAccountError::AcctNameAlreadyExists),
                        Some("unique_email") => Err(UniqueAccountError::EmailAlreadyExists),
                        Some(_) => {
                            panic!("Unknown constraint violated.");
                        }
                    }
                }
            }
        })
    }
}
#[derive(Debug)]
pub struct AccountPasswordInsert(Response<u64>);

impl AccountPasswordInsert {
    pub fn try_recv(&self) -> Result<Result<(), ()>, TryRecvError> {
        Ok(match self.0.try_recv()? {
            Ok(_) => Ok(()),
            Err(e) => { dbg!(e); Err(()) }
        })
    }
}

impl Account {
    pub fn insert_account(database: &Database, acct_name: login::AccountName, email: Option<login::Email>) -> AccountInsert {
        let recv = database.execute(
            "INSERT INTO accounts (name, email) VALUES ($1::TEXT, $2::TEXT)",
            vec![Box::new(acct_name.0), Box::new(email.map(|e| e.0))]
        );

        AccountInsert(recv)
    }

    pub fn insert_password(database: &Database, acct_name: login::AccountName, password: String) -> AccountPasswordInsert {
        let recv = database.execute(
            "INSERT INTO passwords (password, account) VALUES ($2::TEXT, (SELECT id FROM accounts where name = $1::TEXT))",
            vec![Box::new(acct_name.0), Box::new(password)],
        );

        AccountPasswordInsert(recv)
    }
}

pub enum UniqueAccountError {
    AcctNameAlreadyExists,
    EmailAlreadyExists,
}