
class Tree
{
    constructor()
    {

    }

    build_tree(flightlist, flight_select) {
        let tree = document.getElementById("tree");

        const year = new Map();
        const month = new Map();
        
        for (const key in flightlist) {
            if (Object.hasOwnProperty.call(flightlist, key)) {
                const element = flightlist[key];

                let y = year.get(element.date.substring(0, 4));

                if(y == undefined)
                {
                    y = document.createElement("ul");
                    let i = document.createElement("img");
                    i.setAttribute("src","assets/Icons_arrow.svg");
                    let t = document.createElement("div");
                    year.set(element.date.substring(0, 4),y);
                    t.textContent = element.date.substring(0, 4);
                    y.setAttribute("class","year close");
                    t.addEventListener('click',this.year_ckick);
                    t.append(i);
                    y.append(t);
                    tree.append(y);
                }
                let m = month.get(element.date.substring(0, 7));
                if(m == undefined)
                {
                    let i = document.createElement("img");
                    i.setAttribute("src","assets/Icons_arrow.svg");
                    m = document.createElement("ul");
                    let t = document.createElement("div");
                    month.set(element.date.substring(0, 7),m);
                    t.textContent = element.date.substring(5, 7);
                    t.addEventListener('click',this.month_ckick);
                    m.setAttribute("class","month");
                    t.append(i);
                    m.append(t);
                    y.append(m);
                }
            
                let list_element = document.createElement("li");
            
                list_element.addEventListener('click',flight_select,false);
                list_element.flight_id = element.flight_id;
            
                list_element.innerHTML = "<img src=\"assets/Icons_calendar.svg\"><div>"+ element.date +"</div>";
                list_element.innerHTML += "<img src=\"assets/Icons_clock.svg\"><div>"+ format_duration(element.duration)+"</div>";
                if(element.code == '"tri"' || element.code == '"fai"')
                {
                    list_element.innerHTML += "<img src=\"assets/Icons_tri.svg\">"
                }else
                {
                    list_element.innerHTML += "<img src=\"assets/Icons_op.svg\">"
                }
                list_element.innerHTML += "<div>"+ format_distance(element.score,element.code)+"</div>";
            
                m.append(list_element);
                
            }
        }
        Array.from(year.values())[0].setAttribute("class","year");
    }

    year_ckick(evt)
    {
        if(!evt.currentTarget.parentElement.classList.contains("close"))
        {
            evt.currentTarget.parentElement.setAttribute("class","year close");
        }else
        {
            evt.currentTarget.parentElement.setAttribute("class","year");
        }

    }
    month_ckick(evt)
    {
        if(!evt.currentTarget.parentElement.classList.contains("close"))
        {
            evt.currentTarget.parentElement.setAttribute("class","month close");
        }else
        {
            evt.currentTarget.parentElement.setAttribute("class","month");
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