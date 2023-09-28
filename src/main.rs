#[macro_use]

extern crate diesel;

pub mod schema;
pub mod models;

use tera::Tera;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use dotenv::dotenv;
use std::env;

use diesel::prelude::*;
use diesel::pg::PgConnection;

// Librerías para crear una conexión a la BBDD y compartirla en toda la aplicación
use diesel::r2d2::{self, ConnectionManager};
use diesel::r2d2::Pool;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use self::models::{Post, NewPost, NewPostHandler};
use self::schema::posts;
use self::schema::posts::dsl::*;


#[get("/")]
async fn index(pool: web::Data<DbPool>, template_manager: web::Data<tera::Tera>) -> impl Responder{
    let mut conn = pool.get().expect("Problems getting the database");

    match web::block(move || {posts.load::<Post>(&mut conn)}).await {
        Ok(data) => {
            let data = data.unwrap();
            let mut ctx = tera::Context::new();
            ctx.insert("posts", &data);

            HttpResponse::Ok().content_type("text/html").body(
                template_manager.render("index.html", &ctx).unwrap()
            )
        },
        Err(err) => HttpResponse::Ok().body("Error trying to found data")
    }
}

#[get("/blog/{blog_slug}")]
async fn get_post(pool: web::Data<DbPool>, 
    template_manager: web::Data<tera::Tera>,
     blog_slug: web::Path<String>) -> impl Responder{
    let mut conn = pool.get().expect("Problems getting the database");
    
    let url_slug = blog_slug.into_inner();

    match web::block(move || {posts.filter(slug.eq(url_slug)).load::<Post>(&mut conn)}).await {
        Ok(data) => {
            let data = data.unwrap();

            if data.len() == 0 {
                return HttpResponse::NotFound().finish();
            }
            let data = &data[0];

            let mut ctx = tera::Context::new();
            ctx.insert("post", &data);

            HttpResponse::Ok().content_type("text/html").body(
                template_manager.render("post.html", &ctx).unwrap()
            )
        },
        Err(err) => HttpResponse::Ok().body("Error trying to found data")
    }
}

#[post("/new_post")]
async fn new_post(pool: web::Data<DbPool>, item: web::Json<NewPostHandler>) -> impl Responder{
    let mut conn = pool.get().expect("Problems getting the database");
    println!("{:?}", item);

    match web::block(move || {Post::create_post(&mut conn, &item)}).await {
        Ok(data) => {
             HttpResponse::Ok().body(format!("{:?}", data))
        },
        Err(err) => HttpResponse::Ok().body("Error trying to found data")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{

    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DB URL variable not found!");
    let port = env::var("PORT").expect("PORT variable not found!");
    let port: u16 = port.parse().unwrap();

    let connection = ConnectionManager::<PgConnection>::new(db_url);

    let pool = Pool::builder().build(connection).expect("Can't build the pool!");



    HttpServer::new( move|| {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"),"/templates/**/*")).unwrap();
        App::new()
        .service(index)
        .service(new_post)
        .service(get_post)
        .app_data(web::Data::new(pool.clone()))
        .app_data(web::Data::new(tera))
    }).bind(("0.0.0.0", port)).unwrap().run().await

}