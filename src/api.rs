use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct StudentData {
    #[serde(rename = "Id")]
    id: u16,
    stuid: String,
    name: String,
    class: String,
    department: String,
    dt: String,
    openid: String,
    #[serde(rename = "type")]
    student_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TeacherData {
    #[serde(rename = "Id")]
    id: u16,
    tchid: String,
    name: String,
    class: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Lrequest {
    id: u16,
    student: StudentData,
    teacher: TeacherData,
    fromtime: String,
    totime: String,
    reason: String,
    state: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RespBody {
    data: StudentData,
    teacher: TeacherData,
    lrequest: Option<Lrequest>,
    status: String,
}

pub fn check_qr(qr: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let resp = client.get("http://im0o.jiuctd.cn/stu/getStuByQr")
    .query(&[("qr", qr.trim())])
    .header(USER_AGENT, "MicroMessenger")
    .send()?;
    
    println!("{:#?}", resp);
    if resp.status().is_success() {
        let j: RespBody = resp.json()?;
        println!("{:#?}", j);
        if let Some(lrequest) = j.lrequest {
            if lrequest.state == "agree" {
                return Ok(true)
            }
        }
        Ok(false)
    } else {
        println!("Request Failed");
        Ok(false)
    }
}