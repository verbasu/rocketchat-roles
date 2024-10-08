use std::env;
use std::fs::File;
use std::io::Write;
extern crate dotenv;
use dotenv::dotenv;

use crate::models::{
    permission_model::Permission, role_model::Role, room_model::Room, sdui_model::Sdui,
    session_model::Session, user_model::User,
};
use chrono::{TimeZone, Utc};
use mongodb::{
    bson::extjson::de::Error,
    bson::{doc, Document, Bson},
    options::FindOptions,
    sync::{Client, Collection},
    //    results::InsertOneResult,
};

pub struct MongoRepo {
    usercol: Collection<User>,
    rolecol: Collection<Role>,
    permcol: Collection<Permission>,
    sesscol: Collection<Session>,
    roomcol: Collection<Room>,
    sduicol: Collection<Sdui>,
    acolraw: Collection<Document>,
}

impl MongoRepo {
    pub fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGOURI") {
            Ok(v) => v.to_string(),
            Err(_) => "Error loading env variable".to_owned(),
        };
        let cn = match env::var("COLLECTION") {
            Ok(v) => v.to_string(),
            Err(_) => "Error loading env variable".to_owned(),
        };
        let client = Client::with_uri_str(uri).unwrap();
        let db = client.database("rocketchat");
        let usercol: Collection<User> = db.collection("users");
        let rolecol: Collection<Role> = db.collection("rocketchat_roles");
        let permcol: Collection<Permission> = db.collection("rocketchat_permissions");
        let sesscol: Collection<Session> = db.collection("rocketchat_sessions");
        let roomcol: Collection<Room> = db.collection("rocketchat_room");
        let sduicol: Collection<Sdui> = db.collection("rocketchat_settings");
        let acolraw: Collection<Document> = db.collection(&cn);
        MongoRepo {
            usercol,
            permcol,
            sesscol,
            rolecol,
            roomcol,
            sduicol,
            acolraw,
        }
    }

    /*
    pub fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: "test".to_string(),
            username: new_user.username,
            status: new_user.status,
            active: true,
        };
        let user = self
            .col
            .insert_one(new_doc, None)
            .ok()
            .expect("Error creating user");
        Ok(user)
    }
    */

    pub fn get_user(&self, id: &str) -> Result<User, Error> {
        // let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": id};
        let user_detail = self
            .usercol
            .find_one(filter, None)
            .expect("Error getting user's detail");
        Ok(user_detail.unwrap())
    }

    pub fn get_users_by_role(&self, id: &str) -> Result<Vec<User>, Error> {
        let filter = doc! {"roles": { "$in": [id] } };
        let cursors = self
            .usercol
            .find(filter, None)
            .expect("Error getting list of users");
        let users = cursors.map(|doc| doc.unwrap()).collect();
        Ok(users)
    }

    pub fn get_all_users_obj(&self) -> Result<Vec<User>, Error> {
        let trashold = Utc.ymd(2024, 1, 1).and_hms_opt(0, 0, 0);
        let filter = doc! { "$nor": [
            { "roles": { "$exists": false } },
            { "roles": { "$size": 0 } },
            { "roles": { "$in": ["Deactivated"] } },
            { "__rooms": { "$exists": false } },
            { "__rooms": { "$size": 0 } },
            { "active": false },
            { "lastLogin": { "$exists": false } },
            { "lastLogin": { "$lt": trashold } },
            { "emails.verified": false }
        ] };
        let cursors = self
            .usercol
            .find(filter, None)
            .expect("Error getting list of users");
        let users = cursors.map(|doc| doc.unwrap()).collect();
        Ok(users)
    }

    pub fn get_all_rooms(&self) -> Result<Vec<Room>, Error> {
        // let trashold = Utc.ymd(2024, 4, 1).and_hms_opt(0, 0, 0);
        let filter = doc! { "$nor": [
            { "usersCount": { "$exists": false } },
            { "usersCount": { "$lt": 2 } },
            { "msgs": { "$exists": false } },
            { "msgs": { "$lt": 1 } }
        ] };
        let find_options = FindOptions::builder()
            .sort(doc! { "msgs": 1, "usersCount": 1 })
            .build();
        let cursors = self
            .roomcol
            .find(filter, find_options)
            .expect("Error getting list of rooms");
        let rooms = cursors.map(|doc| doc.unwrap()).collect();
        Ok(rooms)
    }

    pub fn get_all_docs(&self) -> Result<Vec<Document>, Error> {
        // let filter = doc! { "$nor": [ { "year": 2023 }, { "year": 2022 }, { "month": 3 }, { "day": 3 } ]};
        let filter = doc! { "u.username": { "$not": {"$regex": "^zabbix.*"} }, "attachments": { "$exists": true } };
        let find_options = FindOptions::builder()
            .limit(1000000)
            .build();
        let mut cursors = self
            .acolraw
            .find(filter, find_options)
            .expect("Error getting list of docs");
        let mut fl = File::create("messages.json").expect("cannot create file");
        while let Some(result) = cursors.next() {
            let doc = result.expect("no messages");
            //let msg: Document = bson::from_bson(Bson::Document(doc)).expect("conversion failed"); 
            let mut s: String = serde_json::to_string(&doc).unwrap();
            // println!("{:?}", s);
            s += "\n";
            fl.write_all(s.as_bytes()).expect("cannot write to file");
        }
        let docs = cursors.map(|doc| doc.unwrap()).collect();
        Ok(docs)
    }

    pub fn get_sessions(&self, id: &str) -> Result<Vec<Session>, Error> {
        let filter = doc! {"userId": id };
        let find_options = FindOptions::builder()
            .sort(doc! { "year": -1, "month": -1, "day": -1 })
            .build();
        let cursors = self
            .sesscol
            .find(filter, find_options)
            .expect("Error getting list of sessions");
        let sessions = cursors.map(|doc| doc.unwrap()).collect();
        Ok(sessions)
    }

    pub fn get_all_roles(&self) -> Result<Vec<Role>, Error> {
        let cursors = self
            .rolecol
            .find(None, None)
            .expect("Error getting list of roles");
        let roles = cursors.map(|doc| doc.unwrap()).collect();
        Ok(roles)
    }

    pub fn get_role_obj(&self, id: &str) -> Result<Role, Error> {
        let filter = doc! {"_id": id};
        let role = self
            .rolecol
            .find_one(filter, None)
            .expect("Error getting role's detail");
        Ok(role.unwrap())
    }

    pub fn get_full_layout(&self) -> Result<Vec<Sdui>, Error> {
        let filter = doc! { "group": "Layout" };
        let cursors = self
            .sduicol
            .find(filter, None)
            .expect("Error getting list of layout elements");
        let layout = cursors.map(|doc| doc.unwrap()).collect();
        Ok(layout)
    }

    pub fn get_all_permissions(&self) -> Result<Vec<Permission>, Error> {
        let filter =
            doc! { "$nor": [ { "roles": { "$exists": false } }, { "roles": { "$size": 0 } } ] };
        let cursors = self
            .permcol
            .find(filter, None)
            .expect("Error getting list of permissions");
        let permissions = cursors.map(|doc| doc.unwrap()).collect();
        Ok(permissions)
    }
}
