import { Component } from '@angular/core';
import axios from 'axios';


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
    
    return axios.post<{}, {}>('http://35.223.240.171:3000/', {
      "user_id": this.user_id,
      "price": this.list[this.chosen] * this.total,
      "amount": this.total
      
  }).then(res => {
    console.log("sent");
    this.loading = false;
    //this.output = 
  });
  }
  
}