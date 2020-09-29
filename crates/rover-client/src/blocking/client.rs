use crate::headers;
use crate::RoverClientError;
use graphql_client::GraphQLQuery;

/// Represents a client for making http requests.
pub struct Client {
    api_key: String,
    client: reqwest::blocking::Client,
    uri: String,
}

impl Client {    
    /// Construct a new [Client] from 2 strings, an `api_key` and an optional `uri`.
    /// if the uri is None, we'll use the default uri for apollo studio
    pub fn new(api_key: String, uri: Option<String>) -> Client {
        let studio_uri = match uri {
            Some(uri) => uri,
            None => "https://graphql.api.apollographql.com/api/graphql".to_string()
        };

        Client {
            api_key,
            client: reqwest::blocking::Client::new(),
            uri: studio_uri
        }
    }

    /// Client method for making a GraphQL request.
    ///
    /// Takes one argument, `variables`. Returns an optional response.
    pub fn post<Q: GraphQLQuery>(
        &self,
        variables: Q::Variables,
    ) -> Result<Option<Q::ResponseData>, RoverClientError> {
        let h = headers::build(&self.api_key)?;
        let body = Q::build_query(variables);

        let response = self
            .client
            .post(&self.uri)
            .headers(h)
            .json(&body)
            .send()
            .map_err(|e| RoverClientError::RequestError { msg: e })?;

        Client::handle_response::<Q>(response)
    }

    fn handle_response<Q: graphql_client::GraphQLQuery>(
        response: reqwest::blocking::Response,
    ) -> Result<Option<Q::ResponseData>, RoverClientError<'static>> {
        let response_body: graphql_client::Response<Q::ResponseData> =
            response
                .json()
                .map_err(|_| RoverClientError::ResponseError {
                    msg: String::from("failed to parse response JSON"),
                })?;

        match response_body.errors {
            Some(errs) => Err(RoverClientError::GraphQLError {
                msg: errs
                    .into_iter()
                    .map(|err| err.message)
                    .collect::<Vec<String>>()
                    .join("\n"),
            }),
            None => Ok(response_body.data),
        }
    }
}
