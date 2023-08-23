# greenlight-rs  -   greenlight  in rust

I port the demo app "greenlight" in [***Let’s Go Further***](https://lets-go-further.alexedwards.net/) to rust using [ ***warp*** ](https://github.com/seanmonstar/warp)  and [Askama template rendering engine](https://github.com/djc/askama/) and [compile-time checked SQLx ](https://github.com/launchbadge/sqlx)

Let's do yet another rust  exercise further

## Building  && Running

### manual

```bash
./scripts/init_db.sh && ./scripts/init_redis.sh
```

```bash
 cargo run --release
```

some cmdline  option:

```bash
./target/release/greenlight  -h
```

```bash
Usage: greenlight [OPTIONS]

Options:
  -l, --log-level <LOG_LEVEL>
          [default: warn]
  -a, --addr <ADDR>
          [default: 0.0.0.0]
  -p, --port <PORT>
          [default: 8000]
      --redis-url <REDIS_URL>
          [default: redis://127.0.0.1:6379]
      --db-dsn <DB_DSN>
          [default: postgres://green:greenpass@localhost:5432/greenlight?sslmode=disable]
      --db-max-conn <DB_MAX_CONN>
          [default: 8]
      --db-connect-timeout <DB_CONNECT_TIMEOUT>
          [default: 2]
      --mail-sender <MAIL_SENDER>
          [default: from@example.com]
      --mail-host <MAIL_HOST>
          [default: sandbox.smtp.mailtrap.io]
      --mail-port <MAIL_PORT>
          [default: 2525]
      --mail-username <MAIL_USERNAME>
          [default: db8ad43072bf5f]
      --mail-password <MAIL_PASSWORD>
          [default: 234fb598e8fa21]
  -h, --help
          Print help
  -V, --version
          Print version
```

### docker-compose

```bash
 docker-compose up -d
```

### some manual  tests

in scripts/init-tables.sql,   preseed  two user   alice and bob:

alice's login credential:   

```json
{"email": "alice@example.com", "password": "alice2green"}
```

bob's login credential:

```json
{"email": "bob@example.com", "password": "(bob2green)"}
```

alice  have   read  and  write  permission

bob   have  read  permission

```bash
curl -i 127.0.0.1:4000/v1/movies
```

```bash
HTTP/1.1 401 Unauthorized
content-type: text/plain; charset=utf-8
content-length: 61
date: Wed, 23 Aug 2023 11:21:30 GMT

{"error":"you must be authenticated to access this resource"}
```

login as  bob,  get  authentication  token:

```bash
curl -i -H Content-Type:application/json -d '{"email": "bob@example.com", "password": "(bob2green)"}' 127.0.0.1:4000/v1/tokens/authentication
```

```bash
HTTP/1.1 201 Created
content-type: application/json
content-length: 105
date: Wed, 23 Aug 2023 11:31:13 GMT

{"authentication_token":{"expiry":"2023-08-24T11:31:13.693974092Z","token":"UGVGRUWXYK7FAMAHLJ3C2S6ETI"}}
```

```bash
curl -X GET -H "Authorization: Bearer UGVGRUWXYK7FAMAHLJ3C2S6ETI" 127.0.0.1:4000/v1/movies | jq .
```

```bash
{
  "metadata": {},
  "movies": []
}
```

try  add  one  movie  entry  using  bob's  token:

```bash
TOKEN="UGVGRUWXYK7FAMAHLJ3C2S6ETI"
BODY='{"title":"Black Panther","year":2018,"runtime":"134 mins","genres":["action","adventure"]}'
curl -i \
-H 'Content-Type: application/json' \
-H "Authorization: Bearer $TOKEN"     \
-d "$BODY" 127.0.0.1:4000/v1/movies
```

```bash
HTTP/1.1 403 Forbidden
content-type: text/plain; charset=utf-8
content-length: 92
date: Wed, 23 Aug 2023 11:47:21 GMT

{"error":"your user account doesn't have the necessary permissions to access this resource"}
```

login  as  alice,  get    token:

```bash
curl -H 'Content-Type:application/json' \
-d '{"email": "alice@example.com", "password": "alice2green"}' \
127.0.0.1:4000/v1/tokens/authentication | jq .
```

```bash
{
  "authentication_token": {
    "expiry": "2023-08-24T11:54:59.351857643Z",
    "token": "72LRYIFWY63QIDRLHOTUYXYLCQ"
  }
}
```

add  some  movie  entries   using   alice's  token:

```bash
TOKEN="72LRYIFWY63QIDRLHOTUYXYLCQ"

BODY='{"title":"Black Panther","year":2018,"runtime":"134 mins","genres":["action","adventure"]}'
curl -sS \
-H 'Content-Type: application/json' \
-H "Authorization: Bearer $TOKEN"     \
-d "$BODY" 127.0.0.1:4000/v1/movies

BODY='{"title":"Deadpool","year":2016, "runtime":"108 mins","genres":["action","comedy"]}'
curl -sS \
-H 'Content-Type: application/json' \
-H "Authorization: Bearer $TOKEN"     \
-d "$BODY" 127.0.0.1:4000/v1/movies


BODY='{"title":"The Breakfast Club","year":1986, "runtime":"96 mins","genres":["drama"]}'
curl -sS \
-H 'Content-Type: application/json' \
-H "Authorization: Bearer $TOKEN"     \
-d "$BODY" 127.0.0.1:4000/v1/movies

BODY='{"title":"Moana","year":2016,"runtime":"107 mins", "genres":["animation","adventure"]}'
curl -sS \
-H 'Content-Type: application/json' \
-H "Authorization: Bearer $TOKEN"     \
-d "$BODY" 127.0.0.1:4000/v1/movies

BODY='{"title":"The Shawshank Redemption","year":1990,"runtime":"142 mins", "genres":["drama"]}'
curl -sS \
-H 'Content-Type: application/json' \
-H "Authorization: Bearer $TOKEN"     \
-d "$BODY" 127.0.0.1:4000/v1/movies
```

movie  list:

```bash
curl -X GET -H "Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ" 127.0.0.1:4000/v1/movies |jq .
```

```bash
{
  "metadata": {
    "current_page": 1,
    "first_page": 1,
    "last_page": 1,
    "page_size": 20,
    "total_records": 5
  },
  "movies": [
    {
      "genres": [
        "action",
        "adventure"
      ],
      "id": 1,
      "runtime": "134 mins",
      "title": "Black Panther",
      "version": 1,
      "year": 2018
    },
    {
      "genres": [
        "action",
        "comedy"
      ],
      "id": 2,
      "runtime": "108 mins",
      "title": "Deadpool",
      "version": 1,
      "year": 2016
    },
    {
      "genres": [
        "drama"
      ],
      "id": 3,
      "runtime": "96 mins",
      "title": "The Breakfast Club",
      "version": 1,
      "year": 1986
    },
    {
      "genres": [
        "animation",
        "adventure"
      ],
      "id": 4,
      "runtime": "107 mins",
      "title": "Moana",
      "version": 1,
      "year": 2016
    },
    {
      "genres": [
        "drama"
      ],
      "id": 5,
      "runtime": "142 mins",
      "title": "The Shawshank Redemption",
      "version": 1,
      "year": 1990
    }
  ]
}
```

search  by  title:

```bash
curl -X GET -H 'Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ' \
'127.0.0.1:4000/v1/movies?title=club'  | jq . 
```

```bash
{
  "metadata": {
    "current_page": 1,
    "first_page": 1,
    "last_page": 1,
    "page_size": 20,
    "total_records": 1
  },
  "movies": [
    {
      "genres": [
        "drama"
      ],
      "id": 3,
      "runtime": "96 mins",
      "title": "The Breakfast Club",
      "version": 1,
      "year": 1986
    }
  ]
}
```

search by  genres:

```bash
curl -X GET -H "Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ" \
127.0.0.1:4000/v1/movies?genres=drama | jq .
```

```bash
{
  "metadata": {
    "current_page": 1,
    "first_page": 1,
    "last_page": 1,
    "page_size": 20,
    "total_records": 2
  },
  "movies": [
    {
      "genres": [
        "drama"
      ],
      "id": 3,
      "runtime": "96 mins",
      "title": "The Breakfast Club",
      "version": 1,
      "year": 1986
    },
    {
      "genres": [
        "drama"
      ],
      "id": 5,
      "runtime": "142 mins",
      "title": "The Shawshank Redemption",
      "version": 1,
      "year": 1990
    }
  ]
}
```

search by title and genres:

```bash
curl -X GET -H "Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ" \
'127.0.0.1:4000/v1/movies?title=moana&genres=animation,adventure' | jq .
```

```bash
{
  "metadata": {
    "current_page": 1,
    "first_page": 1,
    "last_page": 1,
    "page_size": 20,
    "total_records": 1
  },
  "movies": [
    {
      "genres": [
        "animation",
        "adventure"
      ],
      "id": 4,
      "runtime": "107 mins",
      "title": "Moana",
      "version": 1,
      "year": 2016
    }
  ]
}
```

sort  by  title  ascending: 

```bash
curl -X GET -H "Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ" \
'127.0.0.1:4000/v1/movies?sort=title' | jq .
```

```bash
{
  "metadata": {
    "current_page": 1,
    "first_page": 1,
    "last_page": 1,
    "page_size": 20,
    "total_records": 5
  },
  "movies": [
    {
      "genres": [
        "action",
        "adventure"
      ],
      "id": 1,
      "runtime": "134 mins",
      "title": "Black Panther",
      "version": 1,
      "year": 2018
    },
    {
      "genres": [
        "action",
        "comedy"
      ],
      "id": 2,
      "runtime": "108 mins",
      "title": "Deadpool",
      "version": 1,
      "year": 2016
    },
    {
      "genres": [
        "animation",
        "adventure"
      ],
      "id": 4,
      "runtime": "107 mins",
      "title": "Moana",
      "version": 1,
      "year": 2016
    },
    {
      "genres": [
        "drama"
      ],
      "id": 3,
      "runtime": "96 mins",
      "title": "The Breakfast Club",
      "version": 1,
      "year": 1986
    },
    {
      "genres": [
        "drama"
      ],
      "id": 5,
      "runtime": "142 mins",
      "title": "The Shawshank Redemption",
      "version": 1,
      "year": 1990
    }
  ]
}
```

sort by title descending:

```bash
```bash
curl -X GET -H "Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ" \
'127.0.0.1:4000/v1/movies?sort=-title' | jq .
```

```
```bash
{
  "metadata": {
    "current_page": 1,
    "first_page": 1,
    "last_page": 1,
    "page_size": 20,
    "total_records": 5
  },
  "movies": [
    {
      "genres": [
        "drama"
      ],
      "id": 5,
      "runtime": "142 mins",
      "title": "The Shawshank Redemption",
      "version": 1,
      "year": 1990
    },
    {
      "genres": [
        "drama"
      ],
      "id": 3,
      "runtime": "96 mins",
      "title": "The Breakfast Club",
      "version": 1,
      "year": 1986
    },
    {
      "genres": [
        "animation",
        "adventure"
      ],
      "id": 4,
      "runtime": "107 mins",
      "title": "Moana",
      "version": 1,
      "year": 2016
    },
    {
      "genres": [
        "action",
        "comedy"
      ],
      "id": 2,
      "runtime": "108 mins",
      "title": "Deadpool",
      "version": 1,
      "year": 2016
    },
    {
      "genres": [
        "action",
        "adventure"
      ],
      "id": 1,
      "runtime": "134 mins",
      "title": "Black Panther",
      "version": 1,
      "year": 2018
    }
  ]
}
```

show the details of a specific movie:

```bash
curl -X GET -H "Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ" \
'127.0.0.1:4000/v1/movies/5' | jq .
```

```bash
{
  "id": 5,
  "title": "The Shawshank Redemption",
  "year": 1990,
  "runtime": "142 mins",
  "genres": [
    "drama"
  ],
  "version": 1
}
```

partial update  the details of a specific movie:

```bash
curl -X PATCH -H "Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ" \
-H 'Content-Type:application/json' \
-d '{"year": 1994}' '127.0.0.1:4000/v1/movies/5' | jq .
```

```bash
{
  "id": 5,
  "title": "The Shawshank Redemption",
  "year": 1994,
  "runtime": "142 mins",
  "genres": [
    "drama"
  ],
  "version": 2
}
```

delete a specific movie:

```bash
curl -X DELETE  -H "Authorization: Bearer 72LRYIFWY63QIDRLHOTUYXYLCQ" \
'127.0.0.1:4000/v1/movies/5'  | jq .
```

```bash
{
  "message": "movie successfully deleted"
}
```
