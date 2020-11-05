## One way micropayment


### Introduction

We introduce a simple one-way micropayment protocol. There are three steps:

* Sender opens a channel
* (Offline) sender send multiple micropayments to receiver
* Receiver closes a channel

When doing offline micropayment, the sender will sign on the following data:

 |receiver_address|nonce|accumulate_amount|


### Nonce

Each nonce represents a unique "session" id, the sender each time will send the above data with signature. The receiver can only claim the token one time per each nonce. So the receiver will choose the latest and hence the highest value of accumulate amount to claim. When the channel is open, the receiver can claim payments multiple times using different nonce. Once a channel closed and a new channel is opened, all the nonces become available. 

Only the receiver can close the channel. The sender cannot close the channel, but the sender can set an expiration time for this channel. 


### Example

Here is a real world example. Suppose one deeper device A (client) opens the channel with B (server). B will provide network service to A. We assume the expiration window is one week. During this week, A and B communicate to each other once a day. During one day, B provides service to A about one to two hours. During the service window, A(client) will continue to make accumulated micropayment with the **same** nonce. After the service is ending, B chooses to claim payment from A using the latest micropayment. Channel remains open. The next day, A uses a different nonce to make micropayment. Until one week passed, the channel is closed. But B also has the option to close the channel anytime during the week.
