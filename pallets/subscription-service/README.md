# Subscription service

## How to use

The pallet was not tested in real runtime configuration on the substrate node.

However it has basic test coverage, to run tests issue:
`cargo test -p pallet-subscription-service`

## Status and next steps

Provided implementation should be considered as *proof-of-concept* or *skeleton*
for further development of subscription service. At this point some
measurements shall be done (e.g. execution all renewals in on_initialize block
should evaluated).

There is still a lot of to improve:
- Implement more meaningful events (!).
- Registering service_provider/services requires deposit (!).
- Implement fee-free subscribe/cancel (be defensive, not to allow bad transactions to be free) (!).
- Assets pallet shall be integrated for payments and deposits. (!)
- Service provider could be an identity.
- Scheduler pallet may fit, this shall be evaluated.
- Add service removal (but keep all users till the end of the latest subscription period, requires `pending_removal` marker field in ServiceInfo).
- consider adding transactional layer, in renew_subscriptions function, error handling requires additional attention.
- Add RPC calls for utility functions (e.g. `get_renewal_block_for_user`, `is_service_provider_registered`)


Some considerations related to the performance of implementing renewals in `on_initialize` block:
- renew_subscriptions shall check if there are buckets with `bn < now`. If yes, it should be resolved. This relates to the capablity of processing all renewals in `on_initialize`
- renew_subscriptions should terminate if cumulative weight is too high? (size of data stored?)


## Purpose

The middle-men between a service provider and the end user, which allows the
end user to subscribe to the service using crypto tokens (existing ones).

The service allows for recurring subscription payments, one-time subscription
payments (for single period) implemented on chain.

The subscription on-chain could be unstoppable service providing uniform
experience for all subscription services.

The broader idea is to provide the possibility to log into the paid web2
services using public/private keys belonging to the user.

When user logs into the service the service provider checks on-chain if the
given account has entitlements to use the service. 


## Design considerations

Logging to the service is implemented off chain: user provides the signed
"login" message to the service provider, service provider may check the user's
identity.

Checking entitlements may also be implemented off-chain, simply checking the
proof that user's account is in the service's table on chain (this could be
done via light node).

The new pallet would be responsible for:
- accepting the payments and storing the user's account in appropriate data
  structure allowng service provider to check if the subscription fee was made.
- issuing the recurring payment (this part becomes tricky as we need real
  timestamps in runtime)
- allows to cancel recurring subscription,
- manages single-period not recurring subscriptions,
- optionally the user is not charged for the transactions, the cost is covered by service
  provider

Assumption:
- subscription period is counted in blocks


## Pallet's API

- registerServiceProvider(service_provider)
- registerService(service_provider, what, payment_receiver, period); // for service provider
- subscribe(service_provider, what); //user
- cancel(service_provider, what); //user


## Database storage proposal

The storage shall be organized in a way that limits number of entries to be
checked at particular block height, so there is no need to iterate over and check
all the accounts.

To achieve this goal the users' subscriptions will be stored in buckets
organised by `blocknumber`.

At each `blocknumber`, only children of `blocknumber` branch are processed. If
account's balance is high enough, then subscription fee is transferred to
service provider account, otherwise the account is removed from the branch.
After this the branch is renamed to (`blocknumber + service.period`) what can
be read as rescheduling next take-fee operation to the end of new subscription
period.

Example:
(`spXX` - service provider, `sXX` - what service, `aXX` - user account)
```
.
|-- 100
|   |-- a00
|   |   `-- [ {sp00, s00}, {sp03,s08} ]
|   `-- a01
|       `-- [ {sp00, s01} ]
|-- 120
|   |-- a02
|   |   `-- [ {sp01, s01} ]
|   `-- a03
|       `-- [ {sp01, s01} ]
|-- 130
|   `-- a03
|       `-- [ {sp02, s00} ]
|-- 150
|   `-- a00
|       `-- [ {sp01, s00} ]
`-- 200
    `-- a00
        `-- [ {sp02, s00} ]
```
User a00 has four subscriptions:
- `sp00,s00`
- `sp03,s08`
- `sp01,s00`
- `sp02,s00`

User a00's subscriptions will be renewed at the following block heights:
```
@100: sp00,s00 and sp03,s08
@150: sp01,s00
@200: sp02,s00
```

As this layout is not effective for accessing subscriptions by accound_id
(requires traversing all block_number buckets), the cache which maps account_id
to active block numbers is maintained.  For the storage depicted above the
cache would be as follows:
```
  a00: [100,150,200]
  a01: [100]
  a02: [120]
  a03: [120,130]
```

This allows fast access to user's active subscriptions.

