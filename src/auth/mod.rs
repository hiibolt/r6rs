use base64::prelude::*;
use crate::HashMap;

pub struct UbisoftAPI {
    email: String,
    password: String,
    token: String,

    app_id: &str,
    space_ids: HashMap<&str, &str>,5
    headers: HashMap<&str, &str>
}
impl UbisoftAPI {
    fn get_basic_token ( email: String, password: String ) -> Result<String, String> {
        BASE64_STANDARD
            .decode(format!("{}:{}", self.email, self.password))
            .map_err(|_| String::from("Failed to encode email and password!"))?
    }

    fn new ( email: String, password: String ) -> Self {
        let space_ids = HashMap::from([
            ("uplay", "0d2ae42d-4c27-4cb7-af6c-2099062302bb"),
            ("psn", "0d2ae42d-4c27-4cb7-af6c-2099062302bb"),
            ("xbl", "0d2ae42d-4c27-4cb7-af6c-2099062302bb")
        ]);
        let app_id = "e3d5ea9e-50bd-43b7-88bf-39794f4e3d40";
        let token = get_basic_token( email.clone(), password.clone() );

        Self {
            email,
            password,
            token,
            app_id,
            space_ids,
            headers: HashMap::new()
        }
    }

    async fn login ( &mut self ) {
        
    }
}