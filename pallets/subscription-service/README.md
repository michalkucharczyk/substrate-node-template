Subscription service.

Purpose
~~~~~~~

The middle-men between a service provider and the end user, which allows the
end user to subscribe to the service using crypto tokens (existing ones).

The service allows for recurring subscription payments, one-time subscription
payments (for single period).

The idea is to provide the possibility to log into the paid web2 services
using public/private keys belonging to the user.

When user logs into the service the service provider checks if the given
account has entitlements to use the service.


Design considerations
~~~~~~~~~~~~~~~~~~~~~

Logging to the service is implemented off chain: user provides the signed
"login" message to the service provider, service provider may check the user's
identity.

Checking entitlements may also be implemented off-chain, simply checking the
proof that user's account is in the service's table on chain (this could be done via light node).

The new parachain would be responsible for:
- accepting the payments and storing the user's account in appropriate service
  provider's table.

- issuing the recurring payment (this part becomes tricky as we need real time
  in runtime)

- allows to cancel recurring subscription,

- manages single-period not recurring subscriptions,

- the user is not charged for the transactions, the cost is covered by service
  provider

Assumption:
- subscription period is counted in blocks


Pallet's API
~~~~~~~~~~~~

- createSubscription(service_provider, what, payment_receiver, period); // for service provider
- subscribe(service_provider, what); //user
- cancel(service_provider, what); //user

Database storage proposal
~~~~~~~~~~~~~~~~~~~~~~~~~

(spX - service provider, wX - what service, aXX - user account)
.
|-- sp1
|   |-- w1
|   |   |-- 169
|   |   |   `-- a05
|   |   |-- 170
|   |   |   |-- a06
|   |   |   |-- a07
|   |   |   `-- a08
|   |   `-- 199
|   |       `-- a09
|   `-- w2
|       |-- 100
|       |   |-- a00
|       |   |-- a01
|       |   `-- a02
|       |-- 101
|       |   `-- a03
|       `-- 105
|           `-- a04
`-- sp2
    `-- w1
        |-- 110
        |   `-- a10
        |-- 111
        |   `-- a11
        `-- 113
            `-- a12

Storage is organized in a way that limits number of entries to be checked at
particular block height, there is no need to traverse and check all the
accounts.

At each BHNumber, only childs of BHNumber branch are processed. If account's
balance is high enough, then subscription fee is  transferred to service
provider account, otherwise the account is removed from the branch.  After
this the branch is renamed to (BHNumber + period) what can be read as
rescheduling next take-fee operation.

