use actix::prelude::*;
use actix_web::{Error, HttpRequest, HttpResponse};

pub struct Auth {}

pub async fn signIn(req: HttpRequest) {
    // create a challenge

    // send the challenge
}

pub async fn verify(req: HttpRequest) {
    // Verify the challenge

    // safe the user publickey

    // send back authenticated
}
