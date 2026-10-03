const statusBadge = document.getElementById('connection-status');
const countLabel = document.getElementById('packet-count');
const sensorList = document.getElementById('sensor-list');
const sidebar = document.getElementById('sidebar');
const toggleSidebarBtn = document.getElementById('toggle-sidebar');

let packetCount = 0;
// Map to keep track of the HTML elements for each sensor ID
const sensorElements = new Map();

// Sidebar toggle logic
toggleSidebarBtn.addEventListener('click', () => {
    sidebar.classList.toggle('sidebar-collapsed');
});

function connect() {
    const ws = new WebSocket(`ws://${window.location.host}/ws`);

    ws.onopen = () => {
        statusBadge.textContent = 'Connected';
        statusBadge.className = 'px-3 py-1 rounded-full text-sm font-semibold bg-green-500/20 text-green-400 border border-green-500/30';
    };

    ws.onclose = () => {
        statusBadge.textContent = 'Disconnected (Reconnecting...)';
        statusBadge.className = 'px-3 py-1 rounded-full text-sm font-semibold bg-red-500/20 text-red-400 border border-red-500/30';
        setTimeout(connect, 2000); // Reconnect after 2 seconds
    };

    ws.onerror = (err) => {
        console.error('WebSocket Error:', err);
        ws.close();
    };

    ws.onmessage = (event) => {
        const packet = JSON.parse(event.data);
        packetCount++;
        countLabel.textContent = `${packetCount} packets`;

        const sensorIdStr = packet.sensor_id.toString();
        const hexPayload = packet.payload
            .map(b => b.toString(16).padStart(2, '0').toUpperCase())
            .join(' ');

        const now = new Date();
        const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}.${now.getMilliseconds().toString().padStart(3, '0')}`;

        // If we haven't seen this sensor before, create a new card for it
        if (!sensorElements.has(packet.sensor_id)) {
            const sensorCard = document.createElement('div');
            sensorCard.className = 'bg-gray-750 border border-gray-700 rounded-lg p-3 hover:border-gray-500 transition-colors cursor-pointer';
            
            // Layout for the card
            sensorCard.innerHTML = `
                <div class="flex justify-between items-center mb-2">
                    <div class="flex items-center space-x-2">
                        <div class="w-2 h-2 rounded-full bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.8)]"></div>
                        <span class="font-mono font-bold text-blue-400 text-lg">Sensor ${sensorIdStr}</span>
                    </div>
                    <span class="text-xs font-mono text-gray-500 timestamp-label"></span>
                </div>
                <div class="font-mono text-sm text-gray-300 break-all payload-label bg-gray-900 p-2 rounded">
                </div>
            `;
            
            sensorList.appendChild(sensorCard);
            
            // Store references to the elements we need to update frequently
            sensorElements.set(packet.sensor_id, {
                card: sensorCard,
                timeLabel: sensorCard.querySelector('.timestamp-label'),
                payloadLabel: sensorCard.querySelector('.payload-label')
            });
        }

        // Update the card with the latest packet data
        const elements = sensorElements.get(packet.sensor_id);
        elements.timeLabel.textContent = timeStr;
        elements.payloadLabel.textContent = hexPayload;
        
        // Optional: briefly flash the card to indicate new data
        elements.card.classList.add('bg-gray-700');
        setTimeout(() => {
            elements.card.classList.remove('bg-gray-700');
        }, 150);
    };
}

// Start connection
connect();
