# RustLoc

This is a simple web application written in Rust using the Actix-Web framework.

## Features

* Provides public IP address information and geographical location (based on IP) of the client making the request.
* Client's IP address can be passed as a path parameter to get its geographical data.
* The application uses csv files to map IP ranges to countries.

## Installation

### Prerequisites

Ensure you have the following installed on your machine:
* Rust Programming Language (Optimized for version 1.74.1)

### Steps

1. Clone the Git repository: `git clone <repository_url>`
2. Go to the cloned directory: `cd <cloned_directory>`
3. Build the application: `cargo build`
4. Run the application: `cargo run`

By default, the application will listen at `localhost:8080`.

## Usage

* Visit `http://localhost:8080` to get data about your own IP address.
* Visit `http://localhost:8080/{ip_address}` to get data about the provided IP. Replace `{ip_address}` with the IP address you want to look up.