const { invoke } = window.__TAURI__.tauri;

class FlightList{
    constructor(select_callback)
    {
        invoke('history').then((history) => this.build_tree(history,select_callback)).catch((error) => alert(error));
    }

    build_tree(history,select_callback)
    {
        let list = document.getElementById("list");
        const year = new Map();
        const month = new Map();

        let y_ul = document.createElement("ul");

        for (const key in history) {
            if (Object.hasOwnProperty.call(history, key)) {
                const flight = history[key];

                let y = year.get(flight.date.substring(0, 4));

                if(y == undefined)
                {
                    //Create year element
                    y = this.create_ul(flight.date.substring(0, 4));
                    y_ul.appendChild(y);
                    year.set(flight.date.substring(0, 4),y);
                }

                let f_ul = month.get(flight.date.substring(0, 7));

                if(f_ul == undefined)
                {
                    //Create month element
                    f_ul = document.createElement("ul");
                    let m_ul = document.createElement("ul");
                    let m = this.create_ul(flight.date.substring(5, 7));
                    m.appendChild(f_ul);
                    m_ul.appendChild(m);
                    y.appendChild(m_ul);
                    month.set(flight.date.substring(0, 7),f_ul);
                }

                let fl = document.createElement("li");
                fl.setAttribute("class","flight");

                fl.addEventListener('click',flight_select,false);
                fl.flight_id = flight.flight_id;
            
                fl.innerHTML = "<img class=\"icon\" src=\"assets/Icons_calendar.svg\"><div>"+ flight.date +"</div>";
                fl.innerHTML += "<img class=\"icon\" src=\"assets/Icons_clock.svg\"><div>"+ format_duration(flight.duration)+"</div>";
                if(flight.code == '"tri"' || flight.code == '"fai"')
                {
                    fl.innerHTML += "<img class=\"icon\" src=\"assets/Icons_tri.svg\">"
                }else
                {
                    fl.innerHTML += "<img class=\"icon\" src=\"assets/Icons_op.svg\">"
                }
                fl.innerHTML += "<div>"+ format_distance(flight.score,flight.code)+"</div>";

                fl.innerHTML = "<div>"+fl.innerHTML+"</div>";
            
                f_ul.appendChild(fl);
            }
        
        }
        list.appendChild(y_ul);

        function flight_select(evt)
        {
            let prev = document.getElementById('selected');
            if (prev != null){
                prev.removeAttribute('id')
            }
            evt.currentTarget.setAttribute('id','selected');

            invoke('select', {id:parseInt(evt.currentTarget.flight_id)}).then((flight) => select_callback(flight)).catch((error) => alert(error));
        }
    }

    create_ul(text)
    {
        let li = document.createElement("li");
        let div = document.createElement("div");
        let checkbox = document.createElement("input");
        let label = document.createElement("label");
        let img = document.createElement("img");

        img.setAttribute("src","assets/Icons_arrow.svg");
        img.setAttribute("class","arrow");
        checkbox.setAttribute("type","checkbox");

        label.textContent = text;

        div.addEventListener('click',this.ul_onClick);

        div.appendChild(checkbox);
        div.appendChild(label);
        div.appendChild(img);
        li.appendChild(div);
        li.setAttribute("class","close");

        return li;
    }

    ul_onClick(evt)
    {
        console.log("click");
        if(!evt.currentTarget.parentElement.classList.contains("close"))
        {
            evt.currentTarget.parentElement.setAttribute("class","close");
        }else
        {
            evt.currentTarget.parentElement.setAttribute("class","");
        }
    }
}

function format_duration(duration)
{
  let duration_str = "";
  let hour = Math.trunc(duration/60);
  if (hour != 0) 
  {
    duration_str += hour +"h"
  }
  let min = Math.trunc(duration%60);
  if (min < 10)
  {
    duration_str += "0"
  }
  duration_str += min +"min"
  duration_str = duration_str.padEnd(8,'\u2000');
  return duration_str;
}

function format_distance(score,type)
{
  let multiplier = 1000;

  if (type == '"tri"')
  {
    multiplier = 1200;
  }else if(type == '"fai"')
  {
    multiplier = 1400;
  }

  let distance_str = (score/multiplier).toFixed(1) + "km";
  distance_str = distance_str.padEnd(7,'\u2000');
  return distance_str;
}