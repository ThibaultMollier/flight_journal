

class Profile
{
    constructor()
    {
        this.graph = document.getElementById("chart");
        this.svg = document.getElementById("svg");
        this.svg.innerHTML = "";
    }

    draw(profile)
    {
        this.data = CSVToArray(profile);  

        let path = document.createElementNS('http://www.w3.org/2000/svg',"path");
        let maxalt = Math.max(...this.data[1]);
        this.xstep = this.graph.offsetWidth / this.data[1].length;
        this.ystep = this.graph.offsetHeight / (Math.ceil(maxalt/1000)*1000);
        
        let axes = document.createElementNS('http://www.w3.org/2000/svg',"g");
        let alt_line = document.createElementNS('http://www.w3.org/2000/svg',"path");
        let altline_str = "";

        let altstep = 1000;
        if(maxalt <= 2000)
        {
            altstep = 500;
        }
      
        for (let alt = altstep; alt < (Math.ceil(maxalt/1000)*1000); alt+=altstep) {
          let text = document.createElementNS('http://www.w3.org/2000/svg',"text");
          text.setAttribute("x",0);
          text.setAttribute("y",(this.graph.offsetHeight - alt*this.ystep));
          text.textContent = alt + "m";
          altline_str += "M0," + (this.graph.offsetHeight - alt*this.ystep) +"H" + this.graph.offsetWidth;
          axes.appendChild(text);
        }

        this.pointdata_text = document.createElementNS('http://www.w3.org/2000/svg',"text");
        this.pointdata_text.appendChild(document.createElementNS('http://www.w3.org/2000/svg',"tspan"));
        this.pointdata_text.appendChild(document.createElementNS('http://www.w3.org/2000/svg',"tspan"));
        this.pointdata_text.appendChild(document.createElementNS('http://www.w3.org/2000/svg',"tspan"));
        this.pointdata_text.appendChild(document.createElementNS('http://www.w3.org/2000/svg',"tspan"));
        axes.appendChild(this.pointdata_text);
      
        alt_line.setAttribute("d",altline_str);
        axes.setAttribute("stroke","#7c7c7c");
        axes.setAttribute("opacity","0.8");
      
        this.cursor = document.createElementNS('http://www.w3.org/2000/svg',"path");
        axes.appendChild(this.cursor);
        axes.appendChild(alt_line);
        
        let curve_str = "M0.0," + (this.graph.offsetHeight - this.data[1][0]*this.ystep);
      
        for (let index = 1; index < (this.data[1].length-1); index++) {
          curve_str += "L" + index*this.xstep + "," + (this.graph.offsetHeight - this.data[1][index]*this.ystep);
        }
      
        path.setAttribute("d",curve_str);
        path.setAttribute("stroke","#3A00E5");
        path.setAttribute("fill","none");
        path.setAttribute("stroke-width","3");
        this.svg.appendChild(path);        
        this.svg.appendChild(axes);        
    }

    listen(callback_move,callback_scroll)
    {
        this.graph.addEventListener("mousemove", (event) => {
            let i = Math.trunc(event.offsetX/this.xstep);
            let xpos = event.offsetX + 5;
            let ypos = event.offsetY;
            if(event.offsetX > 0.9*this.graph.offsetWidth)
            {
                xpos -= 80;
            }
            this.cursor.setAttribute("d","M" + event.offsetX + ",0V" + this.graph.offsetHeight);
            this.pointdata_text.setAttribute("x",xpos);
            this.pointdata_text.setAttribute("y",ypos);
            this.pointdata_text.setAttribute("stroke","black");
            let time = new Date(this.data[0][i]*1000);

            this.pointdata_text.children[0].textContent = time.getHours() + ":" + time.getMinutes().toString().padStart(2,'0');
            this.pointdata_text.children[0].setAttribute('dy', "1.8rem");
            this.pointdata_text.children[0].setAttribute('x',xpos);
            this.pointdata_text.children[1].textContent = this.data[1][i] + "m";
            this.pointdata_text.children[1].setAttribute('dy', "1.2rem");
            this.pointdata_text.children[1].setAttribute('x',xpos);
            this.pointdata_text.children[2].textContent = Math.round(this.data[2][i]*3.6)  + "km/h";
            this.pointdata_text.children[2].setAttribute('dy',"1.2rem");
            this.pointdata_text.children[2].setAttribute('x',xpos);
            this.pointdata_text.children[3].textContent = (this.data[3][i]*1).toFixed(1)  + "m/s";
            this.pointdata_text.children[3].setAttribute('dy',"1.2rem");
            this.pointdata_text.children[3].setAttribute('x',xpos);
            
            callback_move(this.data[4][i],this.data[5][i]);
        });
        this.graph.addEventListener("wheel", (event) => {
            callback_scroll(event.wheelDelta);
        });
    }

    
}

function CSVToArray( strData, strDelimiter ){
    // Check to see if the delimiter is defined. If not,
    // then default to comma.
    strDelimiter = (strDelimiter || ",");

    // Create a regular expression to parse the CSV values.
    var objPattern = new RegExp(
        (
        // Delimiters.
        "(\\" + strDelimiter + "|\\r?\\n|\\r|^)" +

        // Quoted fields.
        "(?:\"([^\"]*(?:\"\"[^\"]*)*)\"|" +

        // Standard fields.
        "([^\"\\" + strDelimiter + "\\r\\n]*))"
        ),
        "gi"
        );


    // Create an array to hold our this.data. Give the array
    // a default empty first row.
    var arrData = [[]];

    // Create an array to hold our individual pattern
    // matching groups.
    var arrMatches = null;


    // Keep looping over the regular expression matches
    // until we can no longer find a match.
    while (arrMatches = objPattern.exec( strData )){

        // Get the delimiter that was found.
        var strMatchedDelimiter = arrMatches[ 1 ];

        // Check to see if the given delimiter has a length
        // (is not the start of string) and if it matches
        // field delimiter. If id does not, then we know
        // that this delimiter is a row delimiter.
        if (
        strMatchedDelimiter.length &&
        (strMatchedDelimiter != strDelimiter)
        ){

        // Since we have reached a new row of this.data,
        // add an empty row to our this.data array.
        arrData.push( [] );

        }


        // Now that we have our delimiter out of the way,
        // let's check to see which kind of value we
        // captured (quoted or unquoted).
        if (arrMatches[ 2 ]){

        // We found a quoted value. When we capture
        // this value, unescape any double quotes.
        var strMatchedValue = arrMatches[ 2 ].replace(
            new RegExp( "\"\"", "g" ),
            "\""
            );

        } else {

        // We found a non-quoted value.
        var strMatchedValue = arrMatches[ 3 ];

        }


        // Now that we have our value string, let's add
        // it to the this.data array.
        arrData[ arrData.length - 1 ].push( strMatchedValue );
    }

    // Return the parsed this.data.
    return( arrData );
}
