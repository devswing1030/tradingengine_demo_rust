# Trading Engine Demo

This project is a demo of a stock order processing engine.

The author was responsible for developing a stock order processing system that demanded extremely high performance. The order processing module was implemented in C++ and utilized a multi-threaded, multi-stage pipeline architecture to meet performance requirements.

In recent years, Rust has gained significant attention for its excellent memory safety features while delivering performance comparable to C++. This makes it a compelling choice for developers seeking to write safer and more reliable programs.

The primary goal of this demo is to investigate whether Rust can achieve the same level of performance as C++ in developing the aforementioned order processing module while offering substantial improvements in safety.


There is a PowerPoint [presentation](doc/rust%20introduce%20and%20application%20of%20trading%20system.md) 
created by the author after thorough research, designed to introduce Rust to team members and showcase the runtime performance of the demo.

* multi-stage pipeline architecture 
  * Simulated four-stage trading engine pipeline processing:
    * PreProcess: Duplicate order checking and cancellation processing.
    * Check: No processing.
    * Match: Continuous matching.
    * Report: Internal and external binary message generation for reporting.
  * Ensures minimal data access permissions per thread.
  * Data access shared across multi-stage pipelines does not require locking.
![img w:1200](doc/teflow.png)

* Performance Testing
  * Test Data:
    * Randomly generated 50 million orders (30% active cancellations).
  * Test Results:
    * Processing completed in 147.95 seconds, with a throughput of 337.8k orders/second.
    * Approximately 130 million reports generated, with a match rate of around 53%.
  * Meets system performance requirements.