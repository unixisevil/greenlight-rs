use crate::validator::Validator;


pub struct Filter<'a> {
   pub  page: i64,
   pub  page_size: i64,
   pub  sort: &'a str,
   pub  sort_list: &'a [&'a str],
}



impl Filter<'_> {
    pub fn sort_column(&self) -> Option<&str> {
        self.sort_list.iter().find_map(|e| {
            if *e == self.sort {
                Some(e.trim_start_matches('-'))
            } else {
                None
            }
        })
    }

    pub fn sort_direction(&self) -> &'static str {
        if self.sort.starts_with('-') {
            "desc"
        } else {
            "asc"
        }
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }

    pub fn validate(&self, v : &mut Validator) {
        v.check(self.page > 0, "page", "must be greater than zero");
        v.check(
            self.page <= 10_000_000,
            "page",
            "must be a maximum of 10 million",
        );
        v.check(self.page_size > 0, "page_size", "must be greater than zero");
        v.check(
            self.page_size <= 100,
            "page_size",
            "must be a maximum of 100",
        );
        v.check(
            self.sort_list.iter().find(|&&e| e == self.sort).is_some(),
            "sort",
            "invalid sort value",
        );

    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub  struct MetaData {
    #[serde(skip_serializing_if = "is_zero")]
    current_page: i64,

    #[serde(skip_serializing_if = "is_zero")]
    page_size: i64,

    #[serde(skip_serializing_if = "is_zero")]
    first_page: i64,

    #[serde(skip_serializing_if = "is_zero")]
    last_page: i64,

    #[serde(skip_serializing_if = "is_zero")]
    total_records: i64,
}

impl MetaData {
    pub fn calc(total_records: i64, page: i64, page_size: i64) -> Self {
        if total_records == 0 {
            MetaData::default()
        } else {
            MetaData {
                current_page: page,
                page_size,
                first_page: 1,
                last_page:     (total_records as f64  / page_size as f64).ceil() as i64,
                total_records,
            }
        }
    }
}

fn is_zero(num: &i64) -> bool {
    *num == 0
}


