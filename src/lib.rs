#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

pub mod configration;
pub mod logger;

use std::{thread, time::Duration};

use logger::LoggerConfig;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tikv_client::{raw::Client, Config};

use uuid::Uuid;
static TIKV: OnceCell<Client> = OnceCell::new();

#[derive(Clone, Serialize, Deserialize, Debug)]
#[napi(object)]
pub struct TikvConnParams {
  pub tlsclusterenabled: bool,
  pub sslcacerti: String,
  pub sslclientcerti: String,
  pub sslclientkeycerti: String,
  pub host: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[napi(object)]
pub struct BatchResponse {
  pub keys: Vec<String>,
  pub values: Option<Vec<Value>>,
}

#[napi(js_name = "getNextKey")]
pub fn get_next_key() -> String {
  Uuid::new_v4().to_string()
}

#[napi]
pub async fn init_client(tikv_conn_param: Option<TikvConnParams>) -> Result<String, napi::Error> {
  match create_client(tikv_conn_param).await {
    Ok(_res) => Ok(String::from("Client Created")),
    Err(err) => Err(napi::Error::from_reason(format!(
      "Error in Client Creation- {}",
      err.to_string()
    ))),
  }
}

pub async fn create_client(tikv_conn_param: Option<TikvConnParams>) -> Result<Client, napi::Error> {
  let cleint = TIKV.get();
  match cleint {
    Some(_client) => Ok(TIKV.get().unwrap().to_owned()),
    None => {
      let config = Config::default();
      let tls_cluster_enabled = false;
      println!("In tls cluster enable:- {}", tls_cluster_enabled);
      if tls_cluster_enabled {
        let with_security_config = config.to_owned().with_security(
          tikv_conn_param.clone().unwrap().sslcacerti,
          tikv_conn_param.clone().unwrap().sslclientcerti,
          tikv_conn_param.clone().unwrap().sslclientkeycerti,
        );
        let client = Client::new_with_config(
          vec![tikv_conn_param.clone().unwrap().host],
          with_security_config,
        )
        .await;
        match client {
          Ok(client) => {
            let new_client: Client = client.with_atomic_for_cas();
            TIKV.get_or_init(|| new_client.to_owned());
            Ok(new_client)
          }
          Err(error) => Err(napi::Error::from_reason(error.to_string())),
        }
      } else {
        // let security = config.with_security("ca_path", "cert_path", "key_path");
        let client = Client::new_with_config(vec![tikv_conn_param.unwrap().host], config).await;
        match client {
          Ok(client) => {
            let new_client: Client = client.with_atomic_for_cas();
            TIKV.get_or_init(|| new_client.to_owned());
            Ok(new_client)
          }
          Err(error) => Err(napi::Error::from_reason(error.to_string())),
        }
      }
    }
  }
}

#[napi]
pub fn startLogger() {
  // You can use handle to change logger config at runtime
  // just call startLogger() in main.rs and you can use log4rs in all your Project-crate.
  let Global_logs_config = LoggerConfig::create_Global_logs_config();
  let handle = log4rs::init_config(Global_logs_config).unwrap();
}

#[napi(js_name = "getDocument")]
pub async fn get_document(
  key: String,
  withCas: bool,
  project_name: Option<String>,
) -> Result<Value, napi::Error> {
  let client = create_client(None).await;

  match client {
    Ok(client) => {
      let mut new_key = key;
      if !project_name.is_none() && project_name.as_ref().unwrap().len() > 0 {
        let res =
          get_project_level_key_with_global_prefix(project_name.as_ref().unwrap(), &new_key);
        match res {
          Ok(res) => {
            new_key = res;
          }
          Err(error) => {
            return Err(napi::Error::from_reason(error.to_string()));
          }
        }
      }
      let value = client.get(new_key).await; // Returns a `tikv_client::Error` on failure.
      match value {
        Ok(value) => match value {
          Some(value) => {
            let res = String::from_utf8(value)
              .map_err(|e| napi::Error::from_reason(format!("UTF-8 conversion error: {}", e)))?;
            let parsed_json: Value = serde_json::from_str(&res)
              .map_err(|e| napi::Error::from_reason(format!("JSON parsing error: {}", e)))?;
            if withCas {
              return Ok(json!({
                  "data" : parsed_json,
                  "cas" : parsed_json.to_string()
              }));
            } else {
              Ok(json!(parsed_json))
            }
          }
          None => {
            Err(napi::Error::from_reason("Key Does Not Exits".to_string()))
            // return Err("Key Does Not Exits".to_string());
          }
        },
        Err(error) => {
          return Err(napi::Error::from_reason(error.to_string()));
        }
      }
    }
    Err(error) => return Err(napi::Error::from_reason(error.to_string())),
  }
}

#[napi(js_name = "addDocument")]
pub async fn add_document(
  key: String,
  value: Value,
  project_name: Option<String>,
  updateInES: bool,
  retry: Option<u32>,
) -> Result<String, napi::Error> {
  let client = create_client(None).await;
  let retry = retry.unwrap_or(0);
  match client {
    Ok(client) => {
      let mut new_key = key.to_owned();
      if !project_name.is_none() && project_name.as_ref().unwrap().len() > 0 {
        let res =
          get_project_level_key_with_global_prefix(project_name.as_ref().unwrap(), &new_key);
        match res {
          Ok(res) => {
            new_key = res;
          }
          Err(error) => {
            return Err(napi::Error::from_reason(error.to_string()));
          }
        }
      }
      let client_res = client.put(new_key.to_owned(), value.to_string()).await; // Returns a `tikv_client::Error` on failure.
      match client_res {
        Ok(_res) => {
          return Ok(String::from(
            "New Record Added With Key: ".to_owned() + &key,
          ));
        }
        Err(error) => {
          if retry <= 10 {
            log::info!("Retrying for {} time", retry.to_owned());
            thread::sleep(Duration::from_secs(1));
            if let Err(error) = Box::pin(add_document(
              key.to_owned(),
              value,
              project_name,
              updateInES,
              Some(retry + 1),
            ))
            .await
            {
              return Err(napi::Error::from_reason(error.to_string()));
            } else {
              return Ok(String::from(
                "New Record Added With Key: ".to_owned() + &key,
              ));
            }
          } else {
            log::error!(
              "{}",
              format!(
                "Error in put single record after retry count {} for  key {}",
                retry.to_owned(),
                key.to_owned()
              )
            );
            return Err(napi::Error::from_reason(error.to_string()));
          }
        }
      }
    }
    Err(error) => {
      return Err(napi::Error::from_reason(error.to_string()));
    }
  }
}

#[napi(js_name = "replaceDocument")]
pub async fn replace_document(
  key: String,
  value: Value,
  cas: Option<String>,
  project_name: Option<String>,
  updateInES: bool,
  retry: Option<u32>,
) -> Result<String, napi::Error> {
  let client = create_client(None).await;
  let retry = retry.unwrap_or(1);
  
  match client {
    Ok(client) => {
      let mut new_key = key.to_owned();
      if !project_name.is_none() && project_name.as_ref().unwrap().len() > 0 {
        let res =
          get_project_level_key_with_global_prefix(project_name.as_ref().unwrap(), &new_key);
        match res {
          Ok(res) => {
            new_key = res;
          }
          Err(error) => {
            return Err(napi::Error::from_reason(error.to_string()));
          }
        }
      }
      if !cas.is_none() && cas.to_owned().unwrap().len() > 0 {
        let new_eqa = client
          .compare_and_swap(
            new_key.to_owned(),
            Some(cas.clone().unwrap().as_bytes().to_vec()),
            value.to_string().as_bytes().to_vec(),
          )
          .await;
        match new_eqa {
          Ok((_new_val, _flag)) => {
            if !_flag {
              if retry <= 10 {
                log::info!("Retrying for {} time", retry.to_owned());
                thread::sleep(Duration::from_secs(1));
                Box::pin(replace_document(
                  key.to_owned(),
                  value,
                  cas,
                  project_name,
                  updateInES,
                  Some(retry + 1),
                ))
                .await?;
              } else {
                log::error!(
                  "{}",
                  format!(
                    "Error in put single record after retry count {} for  key {}",
                    retry.to_owned(),
                    key.to_owned()
                  )
                );
                return Err(napi::Error::from_reason("Could not Update".to_string()));
              }
            }
            return Ok(String::from(
              "Record Updated With CAS For Key".to_owned() + &key,
            ));
          }
          Err(error) => {
            return Err(napi::Error::from_reason(error.to_string()));
          }
        }
      } else {
        let client_res = client.put(new_key.to_owned(), value.to_string()).await; // Returns a `tikv_client::Error` on failure.
        match client_res {
          Ok(_res) => {
            return Ok(String::from("New Record Added With Key".to_owned() + &key));
          }
          Err(error) => {
            if retry <= 10 {
              log::info!("Retrying for {} time", retry.to_owned());
              thread::sleep(Duration::from_secs(1));
              if let Err(error) = Box::pin(replace_document(
                key.to_owned(),
                value,
                cas,
                project_name,
                updateInES,
                Some(retry + 1),
              ))
              .await
              {
                return Err(napi::Error::from_reason(error.to_string()));
              } else {
                return Ok(String::from("New Record Added With Key".to_owned() + &key));
              }
            } else {
              log::error!(
                "{}",
                format!(
                  "Error in put single record after retry count {} for  key {}",
                  retry.to_owned(),
                  key.to_owned()
                )
              );
              return Err(napi::Error::from_reason(error.to_string()));
            }
          }
        }
      }
    }
    Err(error) => {
      return Err(napi::Error::from_reason(error.to_string()));
    }
  }
}

#[napi(js_name = "getBatchUsingScan")]
pub async fn get_batch_using_scan(
  start: String,
  end: String,
  batch_size: i32,
  keysOnly: bool,
  project_name: Option<String>,
) -> Result<BatchResponse, napi::Error> {
  let client = create_client(None).await;
  match client {
    Ok(client) => {
      let mut start_key = start;
      let mut end_key = end.to_owned();
      let mut prefix_value = "".to_string();
      if !project_name.is_none() && project_name.as_ref().unwrap().len() > 0 {
        let start_key_with_project =
          get_project_level_key_with_global_prefix(project_name.as_ref().unwrap(), &start_key);
        match start_key_with_project {
          Ok(start_key_with_project) => {
            let end_key_with_project =
              get_project_level_key_with_global_prefix(project_name.as_ref().unwrap(), &end_key);
            match end_key_with_project {
              Ok(mut end_key_with_project) => {
                start_key = start_key_with_project;
                if end.len() == 0 {
                  end_key_with_project.push_str("~");
                }
                end_key_with_project.push_str("\\0");
                end_key = end_key_with_project;
                prefix_value = format!("k{}_", project_name.unwrap().to_lowercase())
              }
              Err(error) => {
                return Err(napi::Error::from_reason(error.to_string()));
              }
            }
          }
          Err(error) => {
            return Err(napi::Error::from_reason(error.to_string()));
          }
        }
      }
      if keysOnly {
        let scan_result = client
          .scan_keys((start_key..=end_key).into_inner(), batch_size as u32)
          .await;
        match scan_result {
          Ok(scan_result) => {
            let string_keys: Vec<String> = scan_result
              .iter()
              .map(|key| {
                String::from_utf8_lossy(key.as_ref().into())
                  .to_string()
                  .replace(&prefix_value, "")
              })
              .collect();
            let res = BatchResponse {
              keys: string_keys,
              values: None,
            };
            return Ok(res);
          }
          Err(error) => return Err(napi::Error::from_reason(error.to_string())),
        }
      } else {
        let scan = client
          .scan((start_key..=end_key).into_inner(), batch_size as u32)
          .await;
        match scan {
          Ok(scan) => {
            let string_keys: Vec<String> = scan
              .iter()
              .map(|key| {
                String::from_utf8_lossy(key.0.as_ref().into())
                  .to_string()
                  .replace(&prefix_value, "")
              })
              .collect();
            let string_values: Result<Vec<Value>, napi::Error> = scan
              .iter()
              .map(|key| {
                let res = String::from_utf8_lossy(&key.1.to_vec()).to_string();
                serde_json::from_str(&res)
                  .map_err(|e| napi::Error::from_reason(format!("JSON parsing error: {}", e)))
              })
              .collect();
            match string_values {
              Ok(values) => {
                let res = BatchResponse {
                  keys: string_keys,
                  values: Some(values),
                };
                return Ok(res);
              }
              Err(error) => {
                return Err(napi::Error::from_reason(
                  "Error while parsing values in kev value pair",
                ));
              }
            }
          }
          Err(error) => return Err(napi::Error::from_reason(error.to_string())),
        }
      }
    }
    Err(error) => {
      return Err(napi::Error::from_reason(error.to_string()));
    }
  }
}

#[napi(js_name = "getBatch")]
pub async fn get_batch(
  keys: Vec<String>,
  project_name: Option<String>,
) -> Result<BatchResponse, napi::Error> {
  let client = create_client(None).await;
  let mut prefix_value = "".to_string();
  let mut new_keys: Result<Vec<String>, napi::Error> = Ok(keys.to_owned());
  match client {
    Ok(client) => {
      if !keys.is_empty() {
        if !project_name.is_none() && project_name.as_ref().unwrap().len() > 0 {
          prefix_value = format!("k{}_", project_name.to_owned().unwrap().to_lowercase());
          new_keys = keys
            .into_iter()
            .map(|key| {
              let res =
                get_project_level_key_with_global_prefix(project_name.as_ref().unwrap(), &key);
              let result = match res {
                Ok(res) => res,
                Err(error) => {
                  return Err(napi::Error::from_reason(format!(
                    "Error: {} while processing key {}",
                    error.to_string(),
                    key.to_owned()
                  )));
                }
              };
              Ok(result)
            })
            .collect();
        }

        match new_keys {
          Ok(new_keys) => {
            let get_batch_res = client.batch_get(new_keys).await;
            match get_batch_res {
              Ok(get_batch_res) => {
                let string_keys: Vec<String> = get_batch_res
                  .iter()
                  .map(|key| {
                    String::from_utf8_lossy(key.0.as_ref().into())
                      .to_string()
                      .replace(&prefix_value, "")
                  })
                  .collect();
                let string_values: Result<Vec<Value>, napi::Error> = get_batch_res
                  .iter()
                  .map(|key| {
                    let res = String::from_utf8_lossy(&key.1.to_vec()).to_string();
                    serde_json::from_str(&res)
                      .map_err(|e| napi::Error::from_reason(format!("JSON parsing error: {}", e)))
                  })
                  .collect();
                match string_values {
                  Ok(values) => {
                    let res = BatchResponse {
                      keys: string_keys,
                      values: Some(values),
                    };
                    return Ok(res);
                  }
                  Err(error) => {
                    return Err(napi::Error::from_reason(
                      "Error while parsing values in kev value pair",
                    ));
                  }
                }
              }
              Err(error) => return Err(napi::Error::from_reason(error.to_string())),
            }
          }
          Err(error) => return Err(error),
        };
      } else {
        return Err(napi::Error::from_reason(
          "Empty Array Without keys found".to_string(),
        ));
      }
    }
    Err(error) => {
      log::error!("Failed to create client: {}", error);
      Err(napi::Error::from_reason(error.to_string()))
    }
  }
}

#[napi(js_name = "deleteDocument")]
pub async fn delete_document(
  key: String,
  project_name: Option<String>,
) -> Result<String, napi::Error> {
  let client = create_client(None).await;
  match client {
    Ok(client) => {
      let mut new_key = key.to_owned();
      if !project_name.is_none() && project_name.as_ref().unwrap().len() > 0 {
        let res =
          get_project_level_key_with_global_prefix(project_name.as_ref().unwrap(), &new_key);
        match res {
          Ok(res) => {
            new_key = res;
          }
          Err(error) => {
            return Err(napi::Error::from_reason(error.to_string()));
          }
        }
      }
      let delete_res = client.delete(new_key.to_owned()).await;
      match delete_res {
        Ok(_delete_res) => {
          return Ok(String::from("Record Deleted With Key: ".to_owned() + &key));
        }
        Err(error) => {
          return Err(napi::Error::from_reason(error.to_string()));
        }
      }
    }
    Err(error) => {
      return Err(napi::Error::from_reason(error.to_string()));
    }
  }
}

#[napi(js_name = "getNextCounter")]
pub async fn get_next_counter(
  key: String,
  project_name: Option<String>,
  initial_counter: Option<u32>,
) -> Result<String, napi::Error> {
  // Attempt to retrieve the document using `get_document`
  let doc = get_document(key.clone(), false, project_name.clone()).await;
  // println!("0 : doc : {:?}", doc);
  match doc {
    Ok(doc) => {
      // println!("1 : doc : {:?}", doc);

      // If `doc` is an object, extract the value you're interested in
      if let Some(initial_value) = initial_counter {
        let initial_value_str = initial_value.to_string();
        let value = serde_json::json!(initial_value); // Convert the initial value to a JSON value
        let res = add_document(
          key.clone(),
          value,
          project_name.clone(),
          false,   // Assuming updateInES is false for this scenario
          Some(0), // Initial retry value
        )
        .await;

        match res {
          Ok(_res) => {
            return Ok(initial_value_str);
          }
          Err(_error) => {
            return Err(napi::Error::from_reason(
              "Error while setting the initial counter value".to_string(),
            ));
          }
        }
      }

      // Handle `doc` if it's a JSON object or an integer
      let counter = if let Some(counter_val) = doc.as_i64() {
        counter_val
      } else if let Some(counter_str) = doc.as_str() {
        counter_str.parse::<i64>().unwrap_or(0)
      } else {
        return Err(napi::Error::from_reason(
          "Error: Invalid format for counter value".to_string(),
        ));
      };

      // println!("counter value : {:?}", counter);
      let new_counter = counter + 1;
      let value = serde_json::json!(new_counter); // Convert the new counter to a JSON value
                                                  // println!("counter value in lib : {:?}", value);
      let res = add_document(
        key.clone(),
        value,
        project_name.clone(),
        false,   // Assuming updateInES is false for this scenario
        Some(0), // Initial retry value
      )
      .await;

      match res {
        Ok(_res) => {
          println!("update counter success full : {:?}", _res);
          return Ok(new_counter.to_string());
        }
        Err(error) => {
          return Err(error);
        }
      }
    }
    Err(_error) => {
      // If the key doesn't exist, initialize it with the initial_counter or default to 1
      // println!("2 : doc : {:?}", _error);
      let counter = initial_counter.unwrap_or(1) as i64;
      let value = serde_json::json!(counter); // Convert the counter to a JSON value
      let res = add_document(
        key.clone(),
        value,
        project_name.clone(),
        false,   // Assuming updateInES is false for this scenario
        Some(0), // Initial retry value
      )
      .await;

      match res {
        Ok(_res) => {
          return Ok(counter.to_string());
        }
        Err(_error) => {
          return Err(napi::Error::from_reason(
            "Error while adding counter document".to_string(),
          ));
        }
      }
    }
  }
}

pub fn get_project_level_key_with_global_prefix(
  project: &str,
  key: &str,
) -> Result<String, napi::Error> {
  if project.trim().is_empty() {
    return Err(napi::Error::from_reason(
      "tikv:project cannot be empty".to_string(),
    ));
  }
  if key.trim().contains("~") {
    return Err(napi::Error::from_reason(
      "tikv:invalid character in key: ~".to_string(),
    ));
  }
  let key_with_project_name = format!("k{}_{}", project.trim().to_lowercase(), key.trim());
  Ok(key_with_project_name)
}

// pub fn caste
#[derive(Debug)]
pub struct ReturnError {
  pub error: String,
}
