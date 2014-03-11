// extern crate http;

use response::Response;
use request::Request;
use http::server::ResponseWriter;
use collections::hashmap::HashMap;
use collections::enum_set::{EnumSet};
use http::status::Status;
use pcre::{CompileOption, StudyOption, ExtraOption, Pcre};
use request;
use pcre;
use sync::RWArc;

// Left off here
// TODO change this to a &mut so that it moves the request.
pub type View = fn (&Request, &mut ResponseWriter); //fn<'a>(&'a Request) -> &'a Response;

//#[deriving(Clone)]
pub struct Route<'r> {
	method : &'r str,
	path : &'r str,
	fptr : View,
}

impl<'r> Route<'r> {
    pub fn call(&self, request: &Request, response: &mut ResponseWriter) {
        println!("Routing calling the function for path [{}]", self.path);
        (self.fptr)(request, response)
    }
}

pub trait Router : Clone {
    fn new(routes: &'static [Route<'static>]) -> ~Router;
    fn route(&self, request: &mut Request, response: &mut ResponseWriter);
    fn reverse(&self, name: &str, vars: Option<HashMap<~str,~str>>) -> Option<&~str>;
}

#[deriving(Clone)]
pub struct RegexRouter {
    routes: &'static [Route<'static>],
    compiled_routes: RWArc<Pcre>,
}

impl Router for RegexRouter {
    pub fn new(routes: &'static [Route<'static>]) -> ~Router {
        ~RegexRouter {
            routes: routes,
            compiled_routes: compile_routes(routes),
        } as ~Router
    }

    fn route(&self, request: &mut request::Request, response: &mut ResponseWriter) {
        // use the massive regex to route
        let uri = request.uri.clone();
        let regex_result = self.compiled_routes.read (
            |re: &Pcre| {re.exec(uri)}
        );

        // TODO: clean up this crazy massive match tree using functions found in Option
        // let resp = match regex_result {
        //     Some(_) => {
        //         // get the mark index
        //         let raw_mark = self.compiled_routes.read (
        //             |re: &Pcre| { re.get_mark() }
        //         );
        //         let index = match raw_mark {
        //             // and convert the string to an int
        //             Some(m) => {println!("mark: {}", m); from_str::<int>(m)},
        //             None => None
        //         };
        //         // if we got an int then we can use that as the index in the route array
        //         match index {
        //             Some(i) => Some(self.routes[i].call(request)),
        //             None => None
        //         }
        //     },
        //     None => None
        // };

        // let res = match resp {
        //     Some(res) => res,
        //     None => Response {status: status::NotFound, content: ~"404 - Not Found"}
        // };

        //regex_result.and_then(|_| {
        match regex_result {
            None => (),
            Some(_) => {
                // get the mark to find the index of the route in the routes
                let mark = self.compiled_routes.read (|re: &Pcre| { re.get_mark() });
                // convert the mark to an int and call the appropriate function
                let index = match mark {
                    Some(m) => from_str::<uint>(m),
                    None() => None,
                };
                match index {
                    Some(i) => {self.routes[i].call(request, response)}
                    None => ()
                };
            }
        };

        let reason = response.status.reason();
        let code = response.status.code();

        let newStatus = Status::from_code_and_reason(code,reason);

        response.status = newStatus;
        // return response.content;
    }

    pub fn reverse(&self, name: &str, vars: Option<HashMap<~str,~str>>) -> Option<&~str> {
        None
    }
}

/// Helper method to build a regex out of all the routes
fn compile_routes(routes : &'static [Route<'static>]) -> RWArc<Pcre> {
    // pure evil unsafeness right here
    let mut regex = ~"(?";
    let mut i : u32 = 0;
    for route in routes.iter() {
        regex.push_str("|");
        // TODO add the method to the regex
        //regex.push_str(route.method.to_owned());
        regex.push_str(route.path.to_owned());
        regex.push_str("(*MARK:");
        regex.push_str(i.to_str());
        regex.push_str(")");
        i += 1;
    }
    regex.push_str(")");

    println!("routing regex: {}", regex);

    // set up the regex
    let mut compile_options: EnumSet<CompileOption> = EnumSet::empty();
    compile_options.add(pcre::Extra);
    // TODO: better error handling if unwrap fails on any of these. 
    // I don't think its appropriate to just fail!() either...
    // Maybe an expect explaining the problem would work?
    let compiled_routes = RWArc::<Pcre>::new(
            Pcre::compile_with_options(regex, &compile_options).unwrap()
        );

    let mut study_options: EnumSet<StudyOption> = EnumSet::empty();
    study_options.add(pcre::StudyJitCompile);
    compiled_routes.write(
        |re: &mut Pcre| { re.study_with_options(&study_options); }
    );

    // set that I am using the extra mark field
    let mut extra_options: EnumSet<ExtraOption> = EnumSet::empty();
    extra_options.add(pcre::ExtraMark);
    compiled_routes.write(
        |re: &mut Pcre| { re.set_extra_options(&extra_options); }
    );
    compiled_routes
}

// impl RegexRouter {
// }

// Oxidize { Conf { Router { .route() .match() .compile() } } }