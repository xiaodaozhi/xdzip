#pragma once
#include "rust/cxx.h"
#include "xdzip-qt-bridge/src/lib.rs.h"
#include <QByteArray>
#include <QObject>
#include <QThread>
#include <functional>
#include <memory>
#include <optional>

// Queue storage is shared, but neither thread retains a pointer to the GUI controller.
struct RuntimeMailbox;
class RuntimeReceiver final : public QThread {
    Q_OBJECT
  public:
    RuntimeReceiver(rust::Box<xdzip::reader::EventReceiver> receiver,
                    std::shared_ptr<RuntimeMailbox> mailbox);
    ~RuntimeReceiver() override;
    static int activeReceivers();
  signals:
    void available();

  protected:
    void run() override;

  private:
    rust::Box<xdzip::reader::EventReceiver> m_receiver;
    std::shared_ptr<RuntimeMailbox> m_mailbox;
};

class RuntimeConnection final : public QObject {
    Q_OBJECT
  public:
    explicit RuntimeConnection(QObject *parent = nullptr);
    ~RuntimeConnection() override;
    xdzip::reader::Submission submit(xdzip::reader::Command command);
    void cancel(quint64 request);
    bool cancelTask(quint64 task);
    void releaseExport(quint64 task);
    bool decideTask(quint64 task, quint64 prompt, quint8 policy, bool all);
    bool submitPassword(quint64 task, quint64 prompt, QByteArray password);
    void shutdown();
    struct Statistics {
        std::size_t peakQueued;
        std::size_t peakBytes;
        std::size_t notifications;
    };
    Statistics statistics() const;
    bool stopped() const {
        return m_stopped;
    }
    // Invoked synchronously during a GUI drain; the borrowed event must not be retained.
    using Consumer = std::function<void(const xdzip::reader::Event &)>;
    void setConsumer(Consumer consumer) {
        m_consumer = std::move(consumer);
    }
  signals:
    void disconnected(bool graceful);
  private slots:
    void drain();

  private:
    std::optional<rust::Box<xdzip::reader::CommandClient>> m_client;
    std::shared_ptr<RuntimeMailbox> m_mailbox;
    Consumer m_consumer;
    bool m_receiverFinished = false;
    bool m_stopped = false;
    bool m_shutdownSeen = false;
    quint64 m_sequence = 0;
};
