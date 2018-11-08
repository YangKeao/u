pub struct UCore {
    pre_edit: String,
    commit_msg: String,
}

impl UCore {
    pub fn new() -> UCore {
        UCore {
            pre_edit: String::from(""),
            commit_msg: String::from(""),
        }
    }
    pub fn process_key(&mut self, key_val: u8, modifiers: u8) {
    }
    pub fn render_pre_edit(&self) -> String {
        format!("{}", self.pre_edit)
    }
    pub fn get_hide(&self) -> bool {
        self.pre_edit.len() == 0
    }
    pub fn commit(&mut self) -> String {
        let return_msg = format!("{}", self.commit_msg);
        self.commit_msg = String::from("");

        return return_msg;
    }
    pub fn should_commit(&self) -> bool {
        self.commit_msg.len() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
