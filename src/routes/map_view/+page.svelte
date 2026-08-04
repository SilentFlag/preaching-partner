<script>
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";

    // TODO: use associative array instead
    let streets_index = [];
    let streets = $state([
        {
            id: 0,
            name: "Street Name",
            addresses: [
                { id: 0, checked: true, number: "Address 1" },
                { id: 1, checked: false, number: "Address 1" },
            ],
        },
    ]);

    /**
     * @type {number[]}
     */
    let address_index = [];
    let addresses_store = [[]];

    let addresses_ascending = 0;

    let map_name = $state("Error: Map not found");
    let map_image = $state("");

    let params = new URLSearchParams(window.location.search);
    let map_id = Number(params.get("map_id"));

    listen("map_data_loaded", (event) => {
        streets = [];
        let map_data = event.payload;
        let details = map_data[0];
        let streets_tmp = map_data[1];
        let addresses_tmp = map_data[2];

        for (let i = 0; i < addresses_tmp.length; i++) {
            let address = addresses_tmp[i];
            let street_id = address.street_id;
            if (!address_index.includes(street_id)) {
                address_index.push(street_id);
                addresses_store.push([]);
            }
            let store_index = address_index.indexOf(street_id);
            // @ts-ignore
            addresses_store[store_index].push(address);
        }

        for (let i = 0; i < streets_tmp.length; i++) {
            let id = streets_tmp[i].id;
            let name = streets_tmp[i].name;

            streets_index.push(id);
            streets.push({
                id: id,
                name: name,
                addresses: addresses_store[address_index.indexOf(id)],
            });
        }

        map_name = details.name;
        map_image = "maps/" + details.image_name;
    });

    let map_data = invoke("get_map_data", { mapId: map_id });

    /**
     * @param {number} id
     * @param {boolean} checked
     */
    async function updateCheckbox(id, checked) {
        console.log("Checkbox " + id.toString() + " is " + checked);
        await invoke('complete_address', { id: id, checked: checked });
    }

    /**
     * @param {number} street_id
     */
    function collapseStreet(street_id) {
        let street_element = document.getElementById(
            street_id.toString() + "street",
        );
        if (street_element?.classList.contains("collapsed")) {
            street_element.classList.remove("collapsed");
        } else {
            street_element?.classList.add("collapsed");
        }
    }

    /**
     * @param {number} filter
     */
    function updateFilter(filter) {
        switch (filter) {
            case 0:
                streets.forEach(street => {
                    street.addresses = addresses_store[address_index.indexOf(street.id)]
                });
                updateOrder(addresses_ascending);
                break;
            case 1:
                streets.forEach(street => {
                    if (street.addresses != undefined && street.addresses.length > 0) {
                        /**
                         * @type {any[]}
                         */
                        let tmp_street_addresses = addresses_store[address_index.indexOf(street.id)]
                        let odd_addresses = tmp_street_addresses.filter(address => {
                            let number = parseInt(address.number)
                            return number % 2 === 1
                        })
                        street.addresses = odd_addresses;
                    }
                });
                updateOrder(addresses_ascending);
                break;
            case 2:
                streets.forEach(street => {
                    if (street.addresses != undefined && street.addresses.length > 0) {
                        /**
                         * @type {any[]}
                         */
                        let tmp_street_addresses = addresses_store[address_index.indexOf(street.id)]
                        let even_addresses = tmp_street_addresses.filter(address => {
                            let number = parseInt(address.number)
                            return number % 2 === 0
                        })
                        street.addresses = even_addresses;
                    }
                });
                updateOrder(addresses_ascending);
                break;
            default:
                // TODO: Log Error
        }
    }

    /**
     * 
     * @param {number} order
     */
    function updateOrder(order) {
        if (order === 0) {
            streets.forEach(street => {
                if (street.addresses != undefined && street.addresses.length > 0) {
                    street.addresses.sort((a, b) => parseInt(a.number) - parseInt(b.number));
                }
            });
            addresses_ascending = 0;
        } else if (order === 1) {
            streets.forEach(street => {
                if (street.addresses != undefined && street.addresses.length > 0) {
                    street.addresses.sort((a, b) => parseInt(b.number) - parseInt(a.number));
                }
            });
            addresses_ascending = 1;
        } else {
            // TODO: Log Error
        }
    }
</script>

<main class="container">
    <nav>
        <a href="/dashboard">
            <svg
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
                ><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g
                    id="SVGRepo_tracerCarrier"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                ></g><g id="SVGRepo_iconCarrier">
                    <path
                        d="M12 12C12 11.4477 12.4477 11 13 11H19C19.5523 11 20 11.4477 20 12V19C20 19.5523 19.5523 20 19 20H13C12.4477 20 12 19.5523 12 19V12Z"
                        stroke="#000000"
                        stroke-width="2"
                        stroke-linecap="round"
                    ></path>
                    <path
                        d="M4 5C4 4.44772 4.44772 4 5 4H8C8.55228 4 9 4.44772 9 5V19C9 19.5523 8.55228 20 8 20H5C4.44772 20 4 19.5523 4 19V5Z"
                        stroke="#000000"
                        stroke-width="2"
                        stroke-linecap="round"
                    ></path>
                    <path
                        d="M12 5C12 4.44772 12.4477 4 13 4H19C19.5523 4 20 4.44772 20 5V7C20 7.55228 19.5523 8 19 8H13C12.4477 8 12 7.55228 12 7V5Z"
                        stroke="#000000"
                        stroke-width="2"
                        stroke-linecap="round"
                    ></path>
                </g></svg
            >
            <span>Dashboard</span>
        </a>

        <a href="/maps">
            <svg
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <g id="SVGRepo_bgCarrier" stroke-width="0"></g><g
                    id="SVGRepo_tracerCarrier"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                ></g><g id="SVGRepo_iconCarrier">
                    <path
                        d="M12 6H12.01M9 20L3 17V4L5 5M9 20L15 17M9 20V14M15 17L21 20V7L19 6M15 17V14M15 6.2C15 7.96731 13.5 9.4 12 11C10.5 9.4 9 7.96731 9 6.2C9 4.43269 10.3431 3 12 3C13.6569 3 15 4.43269 15 6.2Z"
                        stroke="#000000"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    ></path>
                </g></svg
            >
            <span>Maps</span>
        </a>

        <a href="/personal">
            <svg
                height="200px"
                width="200px"
                version="1.1"
                id="Capa_1"
                xmlns="http://www.w3.org/2000/svg"
                xmlns:xlink="http://www.w3.org/1999/xlink"
                viewBox="0 0 311.566 311.566"
                xml:space="preserve"
                fill="#000000"
                ><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g
                    id="SVGRepo_tracerCarrier"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                ></g><g id="SVGRepo_iconCarrier">
                    <g>
                        <path
                            style="fill:#010002;"
                            d="M182.19,28.5l-41.034,27.919c-13.622,9.267-14.756,15.43-2.524,13.772 c12.232-1.659,21.051,3.216,19.709,10.89c-1.349,7.667,8.431,6.128,21.839-3.449l44.286-31.636"
                        ></path>
                        <g>
                            <path
                                style="fill:#010002;"
                                d="M304.439,147.682l-19.828-9.35c3.085-6.814,4.869-13.551,4.869-19.351l0.107-24.309 c6.015-6.134,8.801-13.384,7.733-20.478c-1.295-8.598-8.085-15.753-18.623-19.607l-94.623-34.668 c-18.163-6.665-44.811-3.168-60.659,7.936L12.677,105.472c-12.775,8.95-12.703,21.111-12.674,26.905v72.891 c0,20.079,13.867,42.305,31.63,50.617l75.362,34.566c6.599,3.091,14.243,4.571,22.03,4.571c13.455,0,27.322-4.421,36.905-12.614 l127.22-94.897c0.179-0.131,0.352-0.274,0.519-0.418c7.22-6.283,19.136-18.467,17.793-29.494 C310.955,153.321,308.484,149.758,304.439,147.682z M282.171,173.363l-127.196,94.879c-0.179,0.131-0.352,0.274-0.519,0.418 c-9.821,8.539-28.08,11.086-39.924,5.549L39.17,239.643c-11.528-5.394-21.26-21.141-21.26-34.381v-72.939 c-0.036-5.865,0.453-8.974,5.042-12.19l110.739-77.617c7.417-5.197,18.837-8.115,29.423-8.115c5.358,0,10.502,0.752,14.804,2.327 l94.623,34.668c4.887,1.79,6.868,4.069,7.083,5.472c0.161,1.092-0.537,2.995-2.787,5.269c-1.378,0.65-2.566,1.647-3.443,2.87 L160.124,171.12c-10.573,8.043-29.757,10.269-42.06,4.768l-77.927-33.17c-2.763-1.175-5.937-0.895-8.443,0.77 c-2.506,1.653-4.016,4.457-4.016,7.465v31.535c0,19.953,14.207,42.09,32.549,50.492l44.25,18.951 c18.109,8.312,44.203,4.708,59.084-7.936l100.005-77.581c4.314-3.652,8.312-7.984,11.814-12.638l17.059,8.049 C290.852,164.473,287.636,168.572,282.171,173.363z M271.621,108.832l-0.042,10.108c0,4.392-2.345,11.051-6.271,17.745 c-0.465,0.561-0.865,1.187-1.187,1.874c-0.042,0.084-0.078,0.161-0.113,0.251c-3.174,4.982-7.184,9.851-11.713,13.7L152.29,230.09 c-9.977,8.473-28.45,11.015-40.551,5.472l-44.25-18.951c-11.874-5.442-21.898-21.069-21.898-34.131v-18.002l65.319,27.806 c17.996,8.055,44.376,5,60.051-6.922L271.621,108.832z"
                            ></path>
                        </g>
                    </g>
                </g></svg
            >
            <span>Personal Notes</span>
        </a>

        <a href="/settings">
            <svg
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
                ><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g
                    id="SVGRepo_tracerCarrier"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                ></g><g id="SVGRepo_iconCarrier">
                    <path
                        d="M15 12C15 13.6569 13.6569 15 12 15C10.3431 15 9 13.6569 9 12C9 10.3431 10.3431 9 12 9C13.6569 9 15 10.3431 15 12Z"
                        stroke="#000000"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    ></path>
                    <path
                        d="M12.9046 3.06005C12.6988 3 12.4659 3 12 3C11.5341 3 11.3012 3 11.0954 3.06005C10.7942 3.14794 10.5281 3.32808 10.3346 3.57511C10.2024 3.74388 10.1159 3.96016 9.94291 4.39272C9.69419 5.01452 9.00393 5.33471 8.36857 5.123L7.79779 4.93281C7.3929 4.79785 7.19045 4.73036 6.99196 4.7188C6.70039 4.70181 6.4102 4.77032 6.15701 4.9159C5.98465 5.01501 5.83376 5.16591 5.53197 5.4677C5.21122 5.78845 5.05084 5.94882 4.94896 6.13189C4.79927 6.40084 4.73595 6.70934 4.76759 7.01551C4.78912 7.2239 4.87335 7.43449 5.04182 7.85566C5.30565 8.51523 5.05184 9.26878 4.44272 9.63433L4.16521 9.80087C3.74031 10.0558 3.52786 10.1833 3.37354 10.3588C3.23698 10.5141 3.13401 10.696 3.07109 10.893C3 11.1156 3 11.3658 3 11.8663C3 12.4589 3 12.7551 3.09462 13.0088C3.17823 13.2329 3.31422 13.4337 3.49124 13.5946C3.69158 13.7766 3.96395 13.8856 4.50866 14.1035C5.06534 14.3261 5.35196 14.9441 5.16236 15.5129L4.94721 16.1584C4.79819 16.6054 4.72367 16.829 4.7169 17.0486C4.70875 17.3127 4.77049 17.5742 4.89587 17.8067C5.00015 18.0002 5.16678 18.1668 5.5 18.5C5.83323 18.8332 5.99985 18.9998 6.19325 19.1041C6.4258 19.2295 6.68733 19.2913 6.9514 19.2831C7.17102 19.2763 7.39456 19.2018 7.84164 19.0528L8.36862 18.8771C9.00393 18.6654 9.6942 18.9855 9.94291 19.6073C10.1159 20.0398 10.2024 20.2561 10.3346 20.4249C10.5281 20.6719 10.7942 20.8521 11.0954 20.94C11.3012 21 11.5341 21 12 21C12.4659 21 12.6988 21 12.9046 20.94C13.2058 20.8521 13.4719 20.6719 13.6654 20.4249C13.7976 20.2561 13.8841 20.0398 14.0571 19.6073C14.3058 18.9855 14.9961 18.6654 15.6313 18.8773L16.1579 19.0529C16.605 19.2019 16.8286 19.2764 17.0482 19.2832C17.3123 19.2913 17.5738 19.2296 17.8063 19.1042C17.9997 18.9999 18.1664 18.8333 18.4996 18.5001C18.8328 18.1669 18.9994 18.0002 19.1037 17.8068C19.2291 17.5743 19.2908 17.3127 19.2827 17.0487C19.2759 16.8291 19.2014 16.6055 19.0524 16.1584L18.8374 15.5134C18.6477 14.9444 18.9344 14.3262 19.4913 14.1035C20.036 13.8856 20.3084 13.7766 20.5088 13.5946C20.6858 13.4337 20.8218 13.2329 20.9054 13.0088C21 12.7551 21 12.4589 21 11.8663C21 11.3658 21 11.1156 20.9289 10.893C20.866 10.696 20.763 10.5141 20.6265 10.3588C20.4721 10.1833 20.2597 10.0558 19.8348 9.80087L19.5569 9.63416C18.9478 9.26867 18.6939 8.51514 18.9578 7.85558C19.1262 7.43443 19.2105 7.22383 19.232 7.01543C19.2636 6.70926 19.2003 6.40077 19.0506 6.13181C18.9487 5.94875 18.7884 5.78837 18.4676 5.46762C18.1658 5.16584 18.0149 5.01494 17.8426 4.91583C17.5894 4.77024 17.2992 4.70174 17.0076 4.71872C16.8091 4.73029 16.6067 4.79777 16.2018 4.93273L15.6314 5.12287C14.9961 5.33464 14.3058 5.0145 14.0571 4.39272C13.8841 3.96016 13.7976 3.74388 13.6654 3.57511C13.4719 3.32808 13.2058 3.14794 12.9046 3.06005Z"
                        stroke="#000000"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    ></path>
                </g></svg
            >
            <span>Settings</span>
        </a>
    </nav>

    <header>
        <div>
            <a href="/maps">&larr;</a>
            <h1>{map_name}</h1>
        </div>
    </header>

    <div class="content">
        <img src={map_image} alt="Map" />

        <div class="filters">
            <select id="filter" onchange={e => updateFilter(parseInt(e.currentTarget.value))}>
                <option value="0">All Numbers</option>
                <option value="1">Odd</option>
                <option value="2">Even</option>
            </select>
            <select id="order" onchange={e => updateOrder(parseInt(e.currentTarget.value))}>
                <option value="0">Ascending</option>
                <option value="1">Descending</option>
            </select>
        </div>
        

        <!-- TODO: make these expandable/collapsable -->
        {#each streets as street}
            <div class="street" id="{street.id.toString()}street">
                <button
                    class="title"
                    onclick={(e) => collapseStreet(street.id)}
                >
                    <span style="rotate: 90deg">></span>
                    <h2>{street.name}</h2>
                </button>

                <!-- TODO: change to table -->
                <table>
                    <tbody>
                        {#each street.addresses as address}
                            <tr>
                                <td
                                    ><input
                                        type="checkbox"
                                        id={address.id.toString()}
                                        checked={address.checked}
                                        onclick={(e) =>
                                            updateCheckbox(
                                                address.id,
                                                e.currentTarget.checked,
                                            )}
                                    /></td
                                >
                                <td>{address.number}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/each}
    </div>
</main>

<style>
    @import "./map_view.css";
</style>
