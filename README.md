# Build Your Customer Facing Dashboards 10x Faster

Semantic Layer is an open source platform (written in Rust) to build customer facing dashboards 10x faster. You can directly connect your database to the platform and use ready made APIs to query data and create customer facing dashboards.

You no longer have to write custom APIs for different dashboard components and create/maintain SQL queries for them.

![Web App Reference Architecture-5](https://github.com/arihantparsoya/dashboard-api-layer/assets/15258498/156bdb43-23cf-46d5-a212-9c16f2eab01a)

## Use single API to query data for all your dashboard components

You can use a single API endpoint provided by this project to query data for your dashboard. For example: 

```
curl --location 'http://127.0.0.1:8080/api' \
--header 'Content-Type: application/json' \
--data '{
    "metrics": [
        {
        "field": "orders.subtotal",
        "aggregate_operator": "count"
        },
        {
        "field": "orders.total",
        "aggregate_operator": "sum"
        }
    ],
    "dimensions": [
        {
            "field":"products.category"
        }
    ]
}
'
```

The output of the above request will be as follows:

```
{
    "data": [
        [
            "5061",
            "446835.9692339897",
            "Widget"
        ],
        [
            "4784",
            "404989.686671257",
            "Gizmo"
        ],
        [
            "4939",
            "429618.7213845253",
            "Gadget"
        ],
        [
            "3975",
            "313761.33664894104",
            "Doohickey"
        ]
    ]
}
```


## Running Project

### 1. Add your MySQL database credentials
Enter your MySQL database credentials in the [.env](https://github.com/arihantparsoya/dashboard-api-layer/blob/prod/app/server/.env) file.

### 2. Run the Project

Use docker to run the database
```
docker-compose up --build
```

You can start using the docker container path to query your data.

## Features

* 🚀 Fast APIs with Rust
* Single API for all your dashboard requirements
* Automatically generates and execute SQL queries on your database
* Automatically handles complex table relationships

## Integrations

We currently support MySQL database. We will add integrations with other databases in the future.

## Support and Community

Issues are inevitable. When you have one, our entire team and our active developer community is around to help.

💬 Ask for help on [Discord](https://discord.gg/NA9nkZaQnv)

⚠️ Open an issue right here on [GitHub](https://github.com/arihantparsoya/dashboard-semantic-layer/issues/new/choose)

## How to Contribute

We ❤️ our contributors. We're committed to fostering an open, welcoming, and safe environment in the community.

📕 We expect everyone participating in the community to abide by our [Code of Conduct](https://github.com/arihantparsoya/dashboard-semantic-layer/wiki/Code-of-Conduct). Please read and follow it. 

🤝 If you'd like to contribute, start by reading our [Contribution Guide](https://github.com/arihantparsoya/dashboard-semantic-layer/wiki/Guide-to-Contribution).

Lets build great software together.

## License

This project is available under the [Apache License 2.0](https://github.com/arihantparsoya/dashboard-semantic-layer/blob/prod/LICENSE)

