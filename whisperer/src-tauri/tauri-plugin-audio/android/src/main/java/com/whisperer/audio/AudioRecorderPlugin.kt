package com.whisperer.audio

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.os.Build
import androidx.core.app.ActivityCompat
import app.tauri.annotation.Command
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.concurrent.thread

@TauriPlugin(
    permissions = [
        Permission(
            strings = [
                Manifest.permission.RECORD_AUDIO,
                Manifest.permission.MODIFY_AUDIO_SETTINGS
            ],
            alias = "microphone"
        )
    ]
)
class AudioRecorderPlugin(private val activity: Activity) : Plugin(activity) {
    private var audioRecord: AudioRecord? = null
    private var recordingThread: Thread? = null
    private var isRecording = false
    private val sampleRate = 16000 // 16kHz as per Groq recommendation
    private val channelConfig = AudioFormat.CHANNEL_IN_MONO
    private val audioFormat = AudioFormat.ENCODING_PCM_16BIT
    private var audioData = ByteArrayOutputStream()

    @Command
    fun startRecording(invoke: Invoke) {
        if (!hasPermission("microphone")) {
            requestPermission("microphone", invoke)
            return
        }

        try {
            val bufferSize = AudioRecord.getMinBufferSize(sampleRate, channelConfig, audioFormat)
            audioRecord = AudioRecord(
                MediaRecorder.AudioSource.MIC,
                sampleRate,
                channelConfig,
                audioFormat,
                bufferSize
            )

            audioData = ByteArrayOutputStream()
            isRecording = true
            
            audioRecord?.startRecording()

            recordingThread = thread {
                val buffer = ByteArray(bufferSize)
                while (isRecording) {
                    val read = audioRecord?.read(buffer, 0, bufferSize) ?: 0
                    if (read > 0) {
                        audioData.write(buffer, 0, read)
                    }
                }
            }

            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message)
        }
    }

    @Command
    fun stopRecording(invoke: Invoke) {
        try {
            isRecording = false
            recordingThread?.join()
            audioRecord?.stop()
            audioRecord?.release()
            audioRecord = null

            // Convert PCM data to WAV format
            val wavData = createWavFile(audioData.toByteArray())
            invoke.resolve(wavData)
        } catch (e: Exception) {
            invoke.reject(e.message)
        }
    }

    @Command
    fun pauseRecording(invoke: Invoke) {
        // Android AudioRecord doesn't support pause, so we stop recording
        isRecording = false
        invoke.resolve()
    }

    @Command
    fun resumeRecording(invoke: Invoke) {
        // Resume by restarting the recording thread
        isRecording = true
        recordingThread = thread {
            val bufferSize = AudioRecord.getMinBufferSize(sampleRate, channelConfig, audioFormat)
            val buffer = ByteArray(bufferSize)
            while (isRecording) {
                val read = audioRecord?.read(buffer, 0, bufferSize) ?: 0
                if (read > 0) {
                    audioData.write(buffer, 0, read)
                }
            }
        }
        invoke.resolve()
    }

    private fun createWavFile(pcmData: ByteArray): ByteArray {
        val wavOutput = ByteArrayOutputStream()
        
        // WAV header
        val channels = 1
        val bitsPerSample = 16
        val byteRate = sampleRate * channels * bitsPerSample / 8
        val blockAlign = channels * bitsPerSample / 8
        val dataSize = pcmData.size
        
        // Write WAV header
        wavOutput.write("RIFF".toByteArray())
        wavOutput.write(intToByteArray(36 + dataSize))
        wavOutput.write("WAVE".toByteArray())
        wavOutput.write("fmt ".toByteArray())
        wavOutput.write(intToByteArray(16)) // Subchunk1Size
        wavOutput.write(shortToByteArray(1)) // AudioFormat (PCM)
        wavOutput.write(shortToByteArray(channels.toShort()))
        wavOutput.write(intToByteArray(sampleRate))
        wavOutput.write(intToByteArray(byteRate))
        wavOutput.write(shortToByteArray(blockAlign.toShort()))
        wavOutput.write(shortToByteArray(bitsPerSample.toShort()))
        wavOutput.write("data".toByteArray())
        wavOutput.write(intToByteArray(dataSize))
        wavOutput.write(pcmData)
        
        return wavOutput.toByteArray()
    }

    private fun intToByteArray(value: Int): ByteArray {
        return ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN).putInt(value).array()
    }

    private fun shortToByteArray(value: Short): ByteArray {
        return ByteBuffer.allocate(2).order(ByteOrder.LITTLE_ENDIAN).putShort(value).array()
    }
}