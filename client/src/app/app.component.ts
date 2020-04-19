declare var require: any

import { Component } from '@angular/core';

const fetch = require("node-fetch");

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})

export class AppComponent {
  title: string = 'Demo';
  output: string = '';
  user_id: number = 0;
  chosen: string = '';
  total: number = 0;
  items: Array<string> = [];
  price: Array<any> = [];
  loading: boolean = false;
  list: Object = {}

  ngOnInit(){
    this.list["banan"] = 5;
    this.list["gulrot"] = 10;
    this.items = Object.keys(this.list);
    this.price = Object.values(this.list);

  }

  send(){
    this.loading = true;
    console.log("sending");
    console.log(this.chosen);
    console.log(this.user_id);
    console.log(this.list[this.chosen]);
    console.log(this.list[this.chosen] * this.total);

    const data = {
      "account":this.user_id,
      "amount":this.list[this.chosen] * this.total,
      "user_id":this.user_id,
      "items": [1,2,3,4,5]
  }

    fetch("http://localhost:3000/purchase", {
      method: "POST", 
      mode: "cors",
      headers: {
          'Content-Type': 'application/json',
        },
      body: JSON.stringify(data),
    })
    .then(response => response.text())
    .then(data => {
        console.log(data);
        this.output = data;
    })
    .catch((error) => {
      console.error('Error:', error);
    });
  }
}