# NPoW Mechanism
## Overview:
NPoW (Next-gen Proof-of-Work) is an on-chain proof-of-work consensus algorithm in which nodes get rewarded $DPR for completing Web3 tasks. It is decentralized, much more energy efficient, and more economically sustainable than the staking reward or block reward method. The demand side burns EZC to use the provider’s services, and the EZC credit certificate serves as proof-of-work for the service provider.
## Objective:
To provide a mechanism for incentivization of nodes and maintenance of work on the Deeper-Chain. Using sustainable and harmless tasks to motivate nodes on the network to earn rewards, the plan is to gradually replace the distribution of rewards based on staking, the initial reward distribution method.
## Stakeholders:
Deeper Connect users and application users.
## Principle Description:
After a task or application is distributed to limited/all nodes for execution, the user burns the EZC required by the task, while earning the corresponding \$DPR reward. When a node completes a task, the $DPR reward is distributed to the node that completed the task.
## Detailed design:
The early proof-of-work consensus algorithm used hash power to carry out complex calculations, repeated exhausting processes, and verified the process to reach a proof-of-work consensus. The disadvantage of this is that the completion of the computation does not generate actual value, and the result of the work does not contribute to the benefit of people. In other words, nothing beneficial is actually being accomplished. 

Our proposal: a next generation proof-of-work consensus algorithm which is beneficial to people. Decentralized nodes executing Web3 tasks and burning the corresponding EZCs generate actual value for people and achieve effective use of hardware resources. The randomness and anonymity of task distribution ensures that if cheating occurs, it is difficult for cheaters to control and game rewards. In addition, the effort and cost would make the cheating futile.
### 1. Source of Rewards
Rewards come from the user of the application or services, who use a service to burn the corresponding EZC. After the smart contract receives the service request and there is a sufficient balance of EZC, the task will be sent to those nodes that are willing to carry out the automated task. Burning is used to spend the paid EZC in order to reduce the total supply of EZC. The burned EZC is stored in the task detail history as a receipt.
### 2. Distribution
The service provider (node) will receive the EZC certificate of the service from the smart contract. If there are multiple server providers, the EZC credits of this service will be distributed equally amongst the validated providers. The criteria for determining an effective provider depends on whether random tasks are consistently accepted and successfully completed.
### 3. Incentive Mechanism
It is expected that in the early stages of NPoW there will be insufficient tasks and few participant nodes. In this case, a $DPR subsidy will be used to stimulate the development of the Web3 application ecosystem. 5% of the total rewards will be allocated to NPoW, which is 2,460,000 DPR per day. The proportion of the current allocation is tentatively set as follows:
|  Rewards Method   | Maximum Allocation  |  Total Daily DPR  |
|  ----  | ---- |  ---- |
|  NPoW  | 25%  | 1,025,000 |
|  PoCr  | 60%  | 2,460,000 |
|  Validator  | 15%  | 615,000 |  

When the node reward is insufficient (≤100 DPR), the node will be rewarded based on the daily obtained EZC receipts as their contribution. Assuming that the number of EZCs of node A is m, the total number of EZCs in a day is M, and the maximum daily DPR reward is MaxReward(NPoW)=2,460,000, the following formula can buse used:

$$Reward(A) = MaxReward(NPoW)\times{m\over M}$$

When a node gains enough rewards (>100 DPR), it will no longer be considered as a subsidized node, thus will not be compensated by any $DPR subsidies. In an effort to encourage a node to take on more Web3 tasks, rewards will be redeemed 1:1 | EZC:DPR. 

Notes:
* The maximum reward allocation represents the maximum budget of this task. It does not represent an individual’s reward.
* When the maximum reward allocation of the different tasks is not exhausted, the excess DPR will be put into the treasury.
* The maximum reward allocation of the different tasks may be re-adjusted quarterly based on project development and community feedback.
## Tech Challenges and Workload:
* Reward resources, distribution, and smart contract development takes about 3~4 weeks to complete.
* The development of an on-chain incentive consensus algorithm takes about 2-weeks to complete.

## Disadvantages:
Both Deeper and EVM addresses need to be managed, paired and bound when users burn DPR to obtain EZC.