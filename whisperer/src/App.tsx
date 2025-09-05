import { useState, useEffect } from "react";
import { RecordingControls } from "./components/RecordingControls";
import { RecordingsList } from "./components/RecordingsList";
import { Settings } from "./components/Settings";
import { recordingsApi } from "./api";
import type { Recording } from "./types";
import "./App.css";

function App() {
	const [recordings, setRecordings] = useState<Recording[]>([]);
	const [showSettings, setShowSettings] = useState(false);

	useEffect(() => {
		loadRecordings();
	}, []);

	const loadRecordings = async () => {
		try {
			const list = await recordingsApi.list();
			// Sort by date, newest first
			list.sort((a, b) =>
				new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
			);
			setRecordings(list);
		} catch (error) {
			console.error("Failed to load recordings:", error);
		}
	};

	return (
		<div className="app">
			<header className="app-header">
				<h1>Whisperer</h1>
				<button
					onClick={() => setShowSettings(true)}
					className="btn-icon"
				>
					⚙️
				</button>
			</header>

			<main className="app-main">
				<section className="recording-section">
					<RecordingControls onRecordingComplete={loadRecordings} />
				</section>

				<section className="recordings-section">
					<h2>Recordings</h2>
					<RecordingsList
						recordings={recordings}
						onUpdate={loadRecordings}
					/>
				</section>
			</main>

			{showSettings && (
				<Settings onClose={() => setShowSettings(false)} />
			)}
		</div>
	);
}

export default App;
