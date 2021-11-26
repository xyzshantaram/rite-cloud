# rite-cloud

A cloud backup solution for the [rite](https://github.com/xyzshantaram/rite)
text editor. Written in Rust using Tide.

### Usage

You can try out `rite-cloud` using the instance hosted
[here](https://riteapp.co.in). If you would like to host your own instance of
rite-cloud, read on.

#### Self-hosting

You'll need a [working Rust toolchain](https://rustup.rs/), and `openssl` and
`sqlite3` installed.

Clone the repository and cd into it.

```
git clone https://github.com/xyzshantaram/rite-cloud
cd rite-cloud
```

Since rite uses GitHub for authentication, you will need to
[create a GitHub app](https://docs.github.com/en/developers/apps/building-github-apps/creating-a-github-app)
and generate a client secret. Note down the redirect URL you set here, we'll use
it later.

Once this is done, you'll need to set the following environment variables:

- CLIENT_ID - The client ID of the GitHub app you created in the last step.
- CLIENT_SECRET - The client secret you generated in the last step.
- APP_URL - The local URL where you want your app to be, along with the port
  number. For example: `127.0.0.1:8080`.
- TOKEN_URL - The URL where the app will obtain its access token from. Set it to
  `https://github.com/login/oauth/access_token`.
- AUTH_URL - The URL where the app will use for authorization. Set it to
  `https://github.com/login/oauth/authorize`.
- REDIRECT_URL - The URL that GitHub will redirect to on successful auth. It
  should be the URL of where your app is hosted, followed by the string
  `/auth/github/authorized`. For example, the instance at https://riteapp.co.in
  has the REDIRECT_URL `https://riteapp.co.in/auth/github/authorized`
- TIDE_SECRET - A string to use as the cookie signing secret. MUST be atleast 32
  bytes long and should be cryptographically random to be secure.
- DATABASE_URL - A path to an sqlite db (will be created if it doesn't exist)
  that will be used as the rite database.
- FILE_LIMIT - the maximum size of an upload request.

Finally, run the app with `cargo run --release`. You should be able to open it
by navigating to the APP_URL.

### Acknowledgements

- [async-sqlx-session](async-sqlx-session) used under the terms of the MIT
  license. I've temporarily forked it to fix the version of async-session until
  Tide updates with the correct fixes to the SQLite session middleware.

### Contributing

To contribute code, feature ideas or bug reports, open an issue, or fork the
repo and make a PR.

If you would like to help out with hosting costs or just like using rite-cloud,
you can support me financially using one of the means listed
[here](https://shantaram.xyz/contact/donate.html).
