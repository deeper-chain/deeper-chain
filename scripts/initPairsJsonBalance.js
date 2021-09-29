const data = require('.//initPairs.json');
const arr = Object.values(data.initPairs)
//1000000 000 000 000 000 000 0000
arr.forEach(element => {
  console.log("          [");
  console.log("            \"" + element.address + "\",");
  console.log("            10000000000000000000000000");
  console.log("          ],");

});