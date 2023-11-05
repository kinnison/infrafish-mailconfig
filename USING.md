# Using Infrafish - Mailconfig

The Infrafish mail-config API is present at https://mail.infrafish.uk/api/

The API is a basic JSON interface which uses bearer tokens to authenticate.

There are three distinct components to the API - the APIs which are only
exported to internal services, the APIs which are meant to be used by admins,
and the APIs for everyone who has an account on the Infrafish system.

For now, this document will focus on the last of those.

## Calling the API

The API is, as mentioned, an HTTP/JSON interface. The tool `httpie` is useful
for interfacing with the API. Assuming that your token is stored in a file
called `~/.mailconfig.token` then you can have something in your shell akin
to:

```shell

mailconfig () {
    method=$(echo $1 | tr a-z A-Z)
    shift
    uri=$1
    shift
    https -A bearer -a $(cat ~/.mailconfig.token) \
          ${method} https://mail.infrafish.uk/api/${uri} "$@"
}

```

> **NOTE**: If you're on Infrafish already then there is an equivalent shell
> script in your path. No need to add the above to your `.bashrc` or
> equivalent.

We're going to assume you have such a function in your shell and go from there.
Note: using this kind of interface will cause your token to potentially be
visible in `ps` so you may want to find a better option (or wait for a custom
tool) if this worries you. You could also copy the token off onto your own
computer elsewhere and use the API from there - it is accessible from anywhere.

## A brief skim of the admin and infrastructure APIs

To check if the API is alive you can ping it:

```shell

https GET https://mail.infrafish.uk/api/ping

```

Administration APIs are pretty restricted to user management, we won't bother
documenting those here for now. Pretty much all other admin APIs are just
the same APIs as users might use, but with the ability to set which user to
act as / assign to.

## Token APIs

If you have a token already, which you will need in order to do anything,
then you can list, create, and revoke tokens by using the tokens API

### Showing what tokens you have:

```shell
mailconfig get token/list
```

You will get a response along the lines of:

```json
{
  "tokens": [
    {
      "label": "Initial access token",
      "token": "blahblahblah"
    }
  ],
  "used_token": "blahblahblah",
  "username": "yourname"
}
```

### Creating a new token

```shell
mailconfig post token/create label="Hello World"
```

The response will contain the token:

```json
{
  "token": "blahblahblah"
}
```

### Revoking a token

```shell
mailconfig post token/revoke token=blahblahblah
```

Will give you a response containing the label of the token you erased:

```json
{
  "label": "Hello World"
}
```

You may not revoke tokens you do not own, nor may you revoke the token
you are using to access the API at that point in time.

# Domain, entry, and key APIs

The majority of the time you will be interacting with the domain and domain
entry APIs. If you want to support DKIM and set up things like SPF etc.
then you will use the `key` APIs too - of course that requires DNS setup and
also that you smarthost through infrafish for all your mail, so you may not
want to do that to begin with.

Domains have a number of properties, some of which are optional.

In general, domains can have the following:

| Property            | Default | Meaning                                |
| ------------------- | ------- | -------------------------------------- |
| grey-listing        | false   | Apply greylisting to incoming mail     |
| sender-verify       | true    | Perform sender-verify on incoming mail |
| virus-check         | true    | Run incoming mail through clamav       |
| spamcheck-threshold | 100     | Reject spam scoring more than this     |
| remote-mx           | empty   | Forward unmatched mail to this server  |

Creating domains in the system is an admin operation so we will not cover it
for now.

All domain APIs are rooted at `domain/` so let's explore a little.

## Listing domains you have access to

```shell
mailconfig get domain/list
```

This will produce a list of mail domains you have any access to:

```json
{
  "domains": {
    "my-domain.com": {
      "grey-listing": false,
      "sender-verify": true,
      "spamcheck-threshold": 100,
      "virus-check": true
    },
    "another-domain.com": {
      "grey-listing": true,
      "remote-mx": "my.home-server.com",
      "sender-verify": true,
      "spamcheck-threshold": 150,
      "virus-check": true
    }
  }
}
```

## Altering the domain flags

When setting boolean or number entries you must use `:=` rather than just `=`
to cause `httpie` to send raw values rather than wrappering them up as strings.

```shell
mailconfig post domain/set-flags domain-name=my-domain.com grey-listing:=true
```

The returned value from a `set-flags` is the domain settings just like in
the `domain/list` response:

```json
{
  "grey-listing": true,
  "sender-verify": true,
  "spamcheck-threshold": 100,
  "virus-check": true
}
```

Changes to domains will take up to a few minutes to propagate through to the
mail frontends, so please don't expect these to happen immediately.

## Managing mail domain entries

Mail domain entries are all of the usual suspects - mailboxes, aliases,
logins, etc. These are all managed as "entries" because they share a
namespace. Because of how the mail system is set up, changes to entries
have immediate effect, so be careful.

### Listing entries in your domain

```shell
mailconfig get domain/entry/my-domain.com
```

This will give you a mapping of entries in your domain such as:

```json
{
  "entries": {
    "abuse": {
      "expansion": "myname",
      "kind": "alias"
    },
    "myname": {
      "kind": "account"
    },
    "automation": {
      "kind": "login"
    }
  }
}
```

This shows the three kinds of entries currently supported.

- Aliases expand into their given expansion - comma-separated entries which
  will be qualified by the domain name if no `@` is present.
- Logins are username/password pairs which can log in to send email but
  cannot receive mail - this is useful for home mail servers which need to
  smarthost through Infrafish.
- Accounts are username/password pairs which can both log in and receive
  email.

When interacting with other systems, the "username" is always the full account
address such as `myname@my-domain.com` above.

### Creating a new entry

```shell
mailconfig put domain/entry/my-domain.com kind=alias name=foo expansion=myname
```

You'll get a response which looks like:

```json
{
  "created": "foo@my-domain.com"
}
```

Valid `kind`s are `alias`, `account`, and `login`. All three need a `name`,
and where `alias` needs an `expansion`, the other two expect a `password`.

The system will automatically encode the given password using the argon2id
scheme unless the passed in password starts with `{ARGON2ID}` in which case
it is assumed to already be encoded. Please be careful with this capability.

### Retrieving the details of a specific entry

```shell
mailconfig get domain/entry/my-domain.com/foo
```

This will return JSON which looks like an entry in the list response above:

```json
{
  "expansion": "myname",
  "kind": "alias"
}
```

### Deleting an entry

Be careful with this, there is no "undo"

```shell
mailconfig delete domain/entry/my-domain.com/foo
```

Will return:

```json
{
  "deleted": "foo@my-domain.com"
}
```

Again, be super-careful with this, there is **NO UNDO**.

### Updating an entry

You can adjust entries, for aliases that'd be changing the expansion
and for accounts/logins that'd be changing the password:

#### Resetting a password

```shell
mailconfig post domain/entry/my-domain.com/myname password=newpassword
```

You'll get back:

```json
{
  "updated": "myname@my-domain.com"
}
```

#### Editing aliases

Alias expansions can be entirely replaced with:

```shell
mailconfig post domain/entry/my-domain.com/foo expansion=replacement,expansion
```

Or you can add expansion entries with:

```shell
mailconfig post domain/entry/my-domain/foo add=another
```

Or you can remove expansion entries with:

```shell
mailconfig post domain/entry/my-domain/foo remove=myname
```

In all cases, the response is just like for resetting a password.

## DKIM - Domain keys

If you are always smarthosting through infrafish then you can use DKIM to
increase the deliverability of your email. This is non-trivial but for
completeness, here are the APIs you'll need. They're a little less pleasant
to use, but that is meant to discourage fiddling...

### To list your domain keys

```shell
mailconfig post domain/key/list mail-domain=mydomain.com
```

This returns JSON listing your avaialble keys:

```json
{
  "active": {
    "keytag": "v=DKIM1; k=rsa; p=blahblahblahblah"
  },
  "passive": {}
}
```

In brief, active selectors will result in signatures on outgoing mail, passive
selectors will not. This allows you to prepare and get selectors into your DNS
before you start signing, and it allows for you to retire things from your
DNS before you stop signing with them. Key rollover is a complex thing to
discuss and this is not the place to do so.

### To create a new domain key

```shell
mailconfig post domain/key/create mail-domain=mydomain.com selector=blahblah
```

This will create a new keypair and return something like:

```json
{
  "key": "v=DKIM1; k=rsa; p=MIIIomghugelongstring",
  "signing": false
}
```

Note the `signing: false` means that this is a passive selector.

### To mark a selector as active/passive

```shell
mailconfig post domain/key/set-signing mail-domain=mydomain.com selector=blahblah signing:=true
```

This will return:

```json
{
  "signing": true
}
```

If you set `false` instead then obviously that'll be the other way around.
You can also check by using the `domain/key/list` API.

### Deleting a key

Be careful with this, there is **NO UNDO**.

```shell
mailconfig post domain/key/delete mail-domain=mydomain.com selector=blahblah
```

This will return:

```json
{
  "selector": "blahblah",
  "signing": true
}
```

This lets you confirm which selector was deleted and whether or not it had
been signing when you deleted it. If you remove a signing selector then you
may need to do some emergency DNS updates.
