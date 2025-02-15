# Welcome to Loco :train:

[Loco](https://loco.rs) is a web and API framework running on Rust.

This is the **SaaS starter** which includes a `User` model and authentication based on JWT.
It also include configuration sections that help you pick either a frontend or a server-side template set up for your fullstack server.


## Quick Start

```sh
cargo loco start
```

```sh
$ cargo loco start
Finished dev [unoptimized + debuginfo] target(s) in 21.63s
    Running `target/debug/myapp start`

    :
    :
    :

controller/app_routes.rs:203: [Middleware] Adding log trace id

                      ▄     ▀
                                 ▀  ▄
                  ▄       ▀     ▄  ▄ ▄▀
                                    ▄ ▀▄▄
                        ▄     ▀    ▀  ▀▄▀█▄
                                          ▀█▄
▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄▄▄ ▀▀█
 ██████  █████   ███ █████   ███ █████   ███ ▀█
 ██████  █████   ███ █████   ▀▀▀ █████   ███ ▄█▄
 ██████  █████   ███ █████       █████   ███ ████▄
 ██████  █████   ███ █████   ▄▄▄ █████   ███ █████
 ██████  █████   ███  ████   ███ █████   ███ ████▀
   ▀▀▀██▄ ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀ ██▀
       ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
                https://loco.rs

environment: development
   database: automigrate
     logger: debug
compilation: debug
      modes: server

listening on http://localhost:5150
```

## Full Stack Serving

You can check your [configuration](config/development.yaml) to pick either frontend setup or server-side rendered template, and activate the relevant configuration sections.


## Getting help

Check out [a quick tour](https://loco.rs/docs/getting-started/tour/) or [the complete guide](https://loco.rs/docs/getting-started/guide/).


## Raodmap

- [x] Create/update/get/delete product
- [x] Add slug to each product
- [x] Get product by slug
- [ ] Generate a new model for post(product) meta data
- [ ] Save product meta data

- [ ] Cart management based on woocommerce cart and its new Store Api:

  - [x] Generate a hash based on cart content: session_id:id*qty+id2*qty2... and save it on `commust_cart_hash`
  - [x] Save number of items added 2cart in `commust_cart_items` cookie
  - [x] Generate a session id for each user and save it in `coomust_session_id` cookie use middleware if possible
  - [x] Show cart details
  - [x] Remove item from cart
  - [x] Update cart item quantity
  - [ ] Check product availability(instock)

- [ ] Customer orders:

  - [ ] Generate DB migrations
  - [ ] Save orders
  - [ ] Remove orders
  - [ ] Restrict access/update to orders to user/admin

- [ ] Implement authentication via **Cookies**
- [ ] Replace author_id placeholder with a real implementation
- [ ] 
