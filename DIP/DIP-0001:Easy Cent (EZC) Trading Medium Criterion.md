# Easy Cent(EZC) Trading Medium Criterion
## Overview:
EZC(Easy Cent) stable coin is an on-chain non-transferable and non-tradable credit converted from DPR.The purchase and use of EZC is not investment related which is able to pass the SEC's Howey test.Users can use many types of payment methods to make purchases of EZC,such as credit card,Paypal,Apple Pay,etc.
## Objective:
To provide a medium of exchange for payment of applications and service fees on the Deeper Chain. This helps promote the use of on-chain applications at a stable and predictable price.
## Stakeholders:
Developers and various application users on the Deeper Chain's EVM platform
## Principle Description:
The issuance is a type of stable coin (EZC) which has an exchange rate tethered to the US dollar (1EZC = 0.01 USD). Buying of EZC through the use of a credit card or via burning DPR will be convenient. The DPR exchange rate fluctuates and the amount of EZC exchanged will be adjusted based on DPR's market price.
## Detailed design:
### 1. FIAT Exchange Mechanism
Purchasing with a credit card will trigger the EZC minting process, generating the equivalent amount of EZC into the user's wallet address.
### 2. Burning Mechanism
The DPR market price is dynamically obtained through the oracle machine model to get data from multiple nodes. It will start with Byzantine Fault Tolerant (BFT) voting, weighted average, and then the blockchain will exchange the corresponding stable coin based on DPR/USD price.
### 3. Incentivization Mechanism
* DApp transaction fees will be deducted from a user's EZC balance directly.
* If a user has an insufficient EZC balance, automatic burning of DPR will occur in order to obtain the needed amount of EZC for payment. If a user's DPR balance is insufficient, the user will be reminded that the transaction cannot be completed.
* The system will generate an equal amount DPR rewards. The nodes providing the server will receive DPR rewards from the system immediately. The final reward depend on the number of nodes sharing DPR rewards.
### 4. Limitations
* EZC is not transferable.
* EZC cannot be converted to FIAT currency.
* The exchange limit per account is 100,000 EZC.
## Tech Challenges and Workload:
Challenges: Issuance EZC stablecoins and ensuring transaction with DPR.
* Issuing EZC stablecoins takes about one (1) day.
* Completing the transaction between EZC DPR takes about one to two weeks.
* Completing the Oracle machine price feed and integration testing takes about one to two weeks.
## Disadvantages:
Both Deeper and EVM addresses need to be managed, paired and bound when users burn DPR to obtain EZC.