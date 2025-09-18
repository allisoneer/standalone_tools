import { useState, useEffect } from "react";
import { RecordingControls } from "./components/RecordingControls";
import { RecordingsList } from "./components/RecordingsList";
import { Settings } from "./components/Settings";
import { recordingsApi, audioApi } from "./api";
import type { Recording } from "./types";
import "./App.css";

function App() {
	const [recordings, setRecordings] = useState<Recording[]>([]);
	const [showSettings, setShowSettings] = useState(false);
	const [isDragging, setIsDragging] = useState(false);

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

	const handleDragOver = (e: React.DragEvent) => {
		e.preventDefault();
		e.stopPropagation();
	};

	const handleDragEnter = (e: React.DragEvent) => {
		e.preventDefault();
		e.stopPropagation();
		setIsDragging(true);
	};

	const handleDragLeave = (e: React.DragEvent) => {
		e.preventDefault();
		e.stopPropagation();
		setIsDragging(false);
	};

	const handleDrop = async (e: React.DragEvent) => {
		e.preventDefault();
		e.stopPropagation();
		setIsDragging(false);

		const file = e.dataTransfer.files[0];
		if (file && isAudioFile(file)) {
			try {
				const buffer = await file.arrayBuffer();
				await audioApi.uploadFile(buffer, file.name);
				loadRecordings();
			} catch (error) {
				console.error('Drop upload failed:', error);
				alert(`Upload failed: ${error}`);
			}
		}
	};

	const isAudioFile = (file: File): boolean => {
		const audioExtensions = ['mp3', 'm4a', 'aac', 'wav', 'ogg', 'flac'];
		const ext = file.name.split('.').pop()?.toLowerCase();
		return audioExtensions.includes(ext || '');
	};

	return (
		<div 
			className={`app ${isDragging ? 'dragging' : ''}`}
			onDragOver={handleDragOver}
			onDragEnter={handleDragEnter}
			onDragLeave={handleDragLeave}
			onDrop={handleDrop}
		>
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
			
			{isDragging && (
				<div className="drop-overlay">
					<div className="drop-message">Drop audio file to upload</div>
				</div>
			)}
		</div>
	);
}

export default App;
