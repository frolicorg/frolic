
<p align="center">
<a href="">
  <img src="static/Frolic Icon.png" alt="Frolic Logo" width="350">
</a>
</p>

Frolic is an open source backend service (written in Rust) to build customer facing dashboards 10x faster. You can directly connect your database to the project and use ready made APIs to query data. You no longer have to write custom APIs for different dashboard components and create/maintain SQL queries for them.

You can also use [frolic-react](https://github.com/frolicorg/frolic-react) for your UI along with frolic to create full stack dashboards much faster.

![GitHub contributors](https://img.shields.io/github/contributors/FrolicOrg/Frolic)
[![GitHub issues](https://img.shields.io/github/issues/FrolicOrg/Frolic)](https://github.com/FrolicOrg/Frolic/issues)
[![GitHub stars](https://img.shields.io/github/stars/FrolicOrg/Frolic)](https://github.com/FrolicOrg/Frolic/stargazers)
![GitHub closed issues](https://img.shields.io/github/issues-closed/FrolicOrg/Frolic)
![GitHub pull requests](https://img.shields.io/github/issues-pr-raw/FrolicOrg/Frolic)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/FrolicOrg/Frolic)
[![GitHub license](https://img.shields.io/github/license/FrolicOrg/Frolic)](https://github.com/FrolicOrg/Frolic)
[![Twitter Follow](https://img.shields.io/twitter/follow/FrolicOrg?style=social)](https://twitter.com/FrolicOrg)
<!-- 
![GitHub release (latest by date)](https://img.shields.io/github/v/release/FrolicOrg/Frolic) 
![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/tooljet/tooljet-ce)
-->

![Web App Reference Architecture-2](https://github.com/frolicorg/frolic/assets/15258498/9c0d540e-fdd5-4968-8c6e-ff41a655c873)


## Use single API to query data for all your dashboard components

You can use a single API endpoint provided by this project to query data for your dashboard. 

Sample API Request: 

```curl
curl --location 'http://localhost/api' \
--header 'Content-Type: application/json' \
--data '{
      "metrics": [
        {
          "field": "products.price",
          "aggregate_operator": "sum"
        }
      ],
      "dimensions": [
        {
          "field": "products.category"
        }
      ]
}
'
```

You can pass the metrics you require in the `metrics` field as an array. The `field` of the metric is written in `<table_name>.<column_name>` format. The `aggregate_operator` can be used to specifiy what operation you want to apply on the specified `<table_name>.<column_name>`. 

The `dimensions` field allows you to categorize the metrics returned by the API. To specify the column by which you want to categorize the `metrics`, use the `field` operator and specify the column name in `<table_name>.<column_name>` format.

The data returned by the API will be a list of JSON which contains the dimensions and the attributes specified in the request.

The output of the above request will be as follows:

```json
{
    "data": [
        {
            "products.price": "51",
            "products.category": "Gizmo"
        },
        {
            "products.category": "Doohickey",
            "products.price": "42"
        },
        {
            "products.category": "Gadget",
            "products.price": "53"
        },
        {
            "products.category": "Widget",
            "products.price": "54"
        }
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

* 🚀 Fast and Scalable APIs with Rust
* Single API for all your dashboard requirements
* Automatically generates and execute SQL queries on your database
* Automatically handles complex table relationships
* Caching of API Calls (using [memcached](https://memcached.org))

## Integrations

We currently support MySQL database. We will add integrations with other databases in the future.

## Why Rust?

Rust is much faster and performant compared to other web frameworks. We have build this project using `actix-web`, which is one of the [fastest web frameworks in the world](https://www.techempower.com/benchmarks/#section=data-r21). Checkout the comparison between ExpressJS and Actix-Web [here](https://medium.com/@maxsparr0w/performance-of-node-js-compared-to-actix-web-37f20810fb1a).

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

