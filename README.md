# Loto - NFT Raffle on NEAR Protocol

## Introduction
Imagine a protocol, that will allow you to participate in raffle or even create it; and the prize will be some NFT.
The best thing is decentralization, which will give us fair draw (secure random).

## How that works
* NFT (NEP-171) owner sends the NFT, which he wants to draw, to Loto smart-contract. He also set the rules: how many participants will be in the game and ticket price.
* Now, raffle is opened for everyone, and it shows up in the web-client. To participate you just need to attach correct deposit (ticket price). 
* After the last participant registers, the NFT will be transferred to a random participant, and all the money from the tickets will go to the creator. 

## Disadvantages
This project is just an example, what you can make with NFTs on NEAR Protocol.
The reason is opportunity for the creator to play unfairly. How? Well, he can register himself like a participant too, thus he will earn from tickets and also have a chance to win NFT.
The problem can be solved with KYC, or maybe some privacy preserving KYC with zk &#129488;&#129488;&#129488;