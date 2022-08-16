# README

Rust ActixWeb 4 - Async-GraphQL Starter Project

## Urls

- [playground](https://localhost:8643/playground)
- [graphql endpoint](https://localhost:8643/graphql)

## Launch

```shell
$ cargo install cargo-make
$ makers run
# or
$ cargo run
```
## Add Bearer to Playground

- [Using the GraphQL Playground - WellLine](https://docs.wellline.com/wellline-recipes/timeline-service-graphql-api/using-the-graphql-playground-1)

![image](assets/2022-08-15-23-56-25.png)

add 

```json
{
  "Authorization": "Bearer override-secret-here"
}
```

## Test Query/Mutations

use playground or `client.http`

## Test Subscriptions

use playground
