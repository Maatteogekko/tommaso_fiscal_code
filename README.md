[Programming Mentoring challenge](https://discord.com/channels/1130043756477960256/1130173129684160634/1130590091501379664)

# Description

KeroAI is a leading startup that specializes in behavioral prediction through Machine Learning.

The company is about to sign a huge contract with Banca Tommaso, an Italian Bank that wants to predict when their customers will need financial products.

Before signing the contract, Banca Tommaso sent a sample dataset with their customer data and asked KeroAI to analyze it and divide the customers into clusters based on their behavior and characteristics.

As a Software Developer at KeroAI, you are required to write a script to clean the dataset before sending it to the ML training team.

While analyzing the dataset, you realized that some important fields are missing, such as:

- Age
- Gender
- Place of birth

But you can extract them from the [Fiscal Code](https://en.wikipedia.org/wiki/Italian_fiscal_code)!

So, you need to create two functions: one to validate every Fiscal Code and another to extract the needed information.

# Tasks

- Write a function that, given an Italian Fiscal Code, returns `true` if it is valid and `false` otherwise.
- Write a function that, given an Italian Fiscal Code, returns an object (or map, dictionary, set, etc.) with the following contents:
    ```
    {
        bornOn: Date;
        gender: string; 
        placeOfBirth: {
            countryCode: string;
            countryName: string;
            city: string;
            state: string;
        }
    }
    ```
    
Attached, you can find a [codat.json file](codat.json) that will help you obtain information about the place of birth of the owner of the Fiscal Code.