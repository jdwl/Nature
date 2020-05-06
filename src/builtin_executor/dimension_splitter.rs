use std::collections::HashMap;

use serde::Deserialize;
use serde_json::value::RawValue;

use nature_common::{ConverterParameter, ConverterReturned, default_para_separator, Instance, is_default_para_separator, make_key_and_para};

/// Setting is a json, include the following properties:
/// each you defined dimensions will be output as `Instance.para`
#[derive(Serialize, Deserialize)]
pub struct Setting {
    /// default is "/"
    #[serde(skip_serializing_if = "is_default_para_separator")]
    #[serde(default = "default_para_separator")]
    pub dimension_separator: String,
    /// array of array to store dimension index. for example: [["meta-a",[1,2]],["meta-b",[1,3]]].
    pub wanted_dimension: Vec<(String, Vec<u8>)>,
}

/// Data input format required: Vec<Input>
#[derive(Serialize, Deserialize, Clone)]
struct Output<'a> {
    /// include all dimension, separator is defined in Setting
    key: String,
    /// each split dimension will copy this value.
    #[serde(borrow)]
    value: &'a RawValue,
}

#[derive(Clone)]
struct MiddleResult<'a>(String, Vec<Output<'a>>);

/// Suggestion:
/// - use-upstream-id to avoid result scatter
pub fn dimension_split(para: &ConverterParameter) -> ConverterReturned {
    // check setting
    let set = serde_json::from_str::<Setting>(&para.cfg);
    if let Err(e) = set {
        let msg = format!("setting error : {:?}", e.to_string());
        return ConverterReturned::LogicalError(msg);
    }
    let set = set.unwrap();
    if set.wanted_dimension.len() < 1 {
        return ConverterReturned::LogicalError("wanted_dimension does not defined".to_string());
    }

    // check input content
    let input = serde_json::from_str::<Vec<Output>>(&para.from.content);
    if let Err(e) = input {
        let msg = format!("instance content error : {:?}", e.to_string());
        return ConverterReturned::LogicalError(msg);
    }

    // process split
    let mut buffer: HashMap<String, MiddleResult> = HashMap::new();
    let input = input.unwrap();
    for one in input {
        let keys: Vec<&str> = one.key.split(&set.dimension_separator).collect();
        for s in &set.wanted_dimension {
            let result = make_key_and_para(&keys, &s.1, &set.dimension_separator);
            match result {
                Err(e) => { return ConverterReturned::LogicalError(e.to_string()); }
                Ok(tmp) => {
                    let option = buffer.get(&tmp.0);
                    match option {
                        None => {
                            buffer.insert(tmp.0.to_string(), MiddleResult(s.0.to_string(), vec![make_content(tmp.1, one.value)]));
                        }
                        Some(v) => {
                            let mut v = v.clone();
                            v.1.push(make_content(tmp.1, one.value));
                            buffer.insert(tmp.0.to_string(), v);
                        }
                    };
                }
            }
        }
    }

    // make return
    let mut rtn: Vec<Instance> = vec![];
    for one in buffer {
        let mut ins = Instance::default();
        ins.para = one.0.to_string();
        ins.meta = (one.1).0.to_string();
        ins.content = serde_json::to_string(&(one.1).1).unwrap();
        rtn.push(ins);
    }
    ConverterReturned::Instances(rtn)
}

fn make_content(key: String, value: &RawValue) -> Output {
    Output {
        key,
        value,
    }
}

#[cfg(test)]
mod test {
    use nature_common::{Executor, Protocol};
    use nature_db::RelationSettings;

    use super::*;

    #[test]
    fn split_1() {
        let mut content: Vec<KV> = vec![];
        content.push(KV::new("class5|name1|subject1", 92));
        content.push(KV::new("class5|name1|subject2", 85));
        content.push(KV::new("class5|name1|subject3", 99));
        content.push(KV::new("class5|name2|subject1", 67));
        content.push(KV::new("class5|name2|subject2", 81));
        content.push(KV::new("class5|name2|subject3", 75));
        content.push(KV::new("class2|name1|subject1", 100));
        content.push(KV::new("class2|name1|subject2", 98));
        content.push(KV::new("class2|name1|subject3", 73));

        let mut input_instance = Instance::default();
        input_instance.content = serde_json::to_string(&content).unwrap();

        let dimentions = vec![("B:person/score_temp:1".to_string(), vec![0, 1]), ];
        let setting = Setting {
            dimension_separator: "|".to_string(),
            wanted_dimension: dimentions,
        };

        let para = ConverterParameter {
            from: input_instance,
            last_state: None,
            task_id: "".to_string(),
            master: None,
            cfg: serde_json::to_string(&setting).unwrap(),
        };

        let rtn = dimension_split(&para);
        if let ConverterReturned::Instances(ins) = rtn {
            assert_eq!(ins.len(), 3);
        };
    }

    #[test]
    fn split_2() {
        let mut content: Vec<KV> = vec![];
        content.push(KV::new("class5|name1|subject1", 92));
        content.push(KV::new("class5|name1|subject2", 85));
        content.push(KV::new("class5|name1|subject3", 99));
        content.push(KV::new("class5|name2|subject1", 67));
        content.push(KV::new("class5|name2|subject2", 81));
        content.push(KV::new("class5|name2|subject3", 75));
        content.push(KV::new("class2|name1|subject1", 100));
        content.push(KV::new("class2|name1|subject2", 98));
        content.push(KV::new("class2|name1|subject3", 73));

        let mut input_instance = Instance::default();
        input_instance.content = serde_json::to_string(&content).unwrap();


        let dimentions = vec![("B:subject/class_score_temp:1".to_string(), vec![0, 2]), ];
        let setting = Setting {
            dimension_separator: "|".to_string(),
            wanted_dimension: dimentions,
        };

        let para = ConverterParameter {
            from: input_instance,
            last_state: None,
            task_id: "".to_string(),
            master: None,
            cfg: serde_json::to_string(&setting).unwrap(),
        };

        let rtn = dimension_split(&para);
        if let ConverterReturned::Instances(ins) = rtn {
            assert_eq!(ins.len(), 6);
        };
    }

    #[test]
    fn split_3() {
        let mut content: Vec<KV> = vec![];
        content.push(KV::new("class5|name1|subject1", 92));
        content.push(KV::new("class5|name1|subject2", 85));
        content.push(KV::new("class5|name1|subject3", 99));
        content.push(KV::new("class5|name2|subject1", 67));
        content.push(KV::new("class5|name2|subject2", 81));
        content.push(KV::new("class5|name2|subject3", 75));
        content.push(KV::new("class2|name1|subject1", 100));
        content.push(KV::new("class2|name1|subject2", 98));
        content.push(KV::new("class2|name1|subject3", 73));

        let mut input_instance = Instance::default();
        input_instance.content = serde_json::to_string(&content).unwrap();

        let dimentions = vec![
            ("B:person/score_temp:1".to_string(), vec![0, 1]),
            ("B:subject/class_score_temp:1".to_string(), vec![0, 2]),
        ];

        let setting = Setting {
            dimension_separator: "|".to_string(),
            wanted_dimension: dimentions,
        };

        let para = ConverterParameter {
            from: input_instance,
            last_state: None,
            task_id: "".to_string(),
            master: None,
            cfg: serde_json::to_string(&setting).unwrap(),
        };

        let rtn = dimension_split(&para);
        if let ConverterReturned::Instances(ins) = rtn {
            assert_eq!(ins.len(), 9);
        };
    }

    #[test]
    fn setting_default() {
        let set = Setting {
            dimension_separator: "/".to_string(),
            wanted_dimension: vec![("a".to_string(), vec![1])],
        };
        let json = serde_json::to_string(&set).unwrap();
        let cmp = r#"{"wanted_dimension":[["a",[1]]]}"#;
        assert_eq!(json, cmp);
        let set = serde_json::from_str::<Setting>(&json).unwrap();
        assert_eq!(set.dimension_separator, "/")
    }

    #[test]
    fn setting_json_generator() {
        let set = Setting {
            dimension_separator: "/".to_string(),
            wanted_dimension: vec![
                ("B:score/trainee/original:1".to_string(), vec![0, 1]),
                ("B:score/subject/original:1".to_string(), vec![0, 2]),
            ],
        };
        let json = serde_json::to_string(&set).unwrap();
        let exe = Executor {
            protocol: Protocol::BuiltIn,
            url: "dimensionSplit".to_string(),
            settings: json,
        };
        let mut rela = RelationSettings::default();
        rela.executor = Some(exe);
        let json = serde_json::to_string(&rela).unwrap();
        let _rela = serde_json::from_str::<RelationSettings>(&json).unwrap();
        assert_eq!(json, r#"{"executor":{"protocol":"builtIn","url":"dimensionSplit","settings":"{\"wanted_dimension\":[[\"B:score/trainee/original:1\",[0,1]],[\"B:score/subject/original:1\",[0,2]]]}"}}"#);
    }

    #[derive(Serialize)]
    struct KV {
        pub key: String,
        pub value: i32,
    }

    impl KV {
        pub fn new(key: &str, value: i32) -> Self {
            KV {
                key: key.to_string(),
                value,
            }
        }
    }
}